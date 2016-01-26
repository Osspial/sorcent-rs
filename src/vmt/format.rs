use vmt::lexer::Token;
use vmt::error::{VMTResult, VMTError};

use std::fmt;
use std::default;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    ShaderType,
    ShaderBlockStart,
    ShaderBlockEnd,

    ShaderParamType,
    ShaderParamValue,

    // Fallback related states
    FallBlockStart,
    FallBlockEnd,
    FallParamType,
    FallParamValue,

    Default
}

#[derive(Debug)]
pub struct Shader {
    s_type: RSlice,
    parameters: Vec<Parameter>,
    fallbacks: Vec<Fallback>,
    //pub proxies: Vec<Proxy<'s>>
}


impl Shader {
    /// element_lens: a vector of the lengths of each slice contained in element_str
    pub fn from_raw_parts<'s>(tokens: &Vec<Token>, element_str: &'s str, element_lens: &Vec<usize>) -> VMTResult<Shader> {
        let mut shader_type: &'s str = "";
        // Most materials don't have any fallbacks, so in most cases
        // we can avoid a heap allocation.
        let mut fallbacks: Vec<Fallback> = Vec::new();
        let mut parameters: Vec<Parameter> = Vec::with_capacity(16);
        let mut state = State::Default;

        let mut fallback_temp: Fallback = Default::default();

        // What number token we're on
        let mut ti = 0;
        // Where in the element string we are
        let mut elc = 0;

        // Temporary storage locations for parameter types
        let mut parameter_type: &'s str = "";
        
        for t in tokens {
            match *t {
                Token::Start |
                Token::End      => (),
                _   => {
                    match state {
                        // If the file just been loaded, the first block type
                        // is going to correspond to a shader.
                        State::Default          => {
                            match *t {
                                Token::BlockType(_) => {
                                    state = State::ShaderType;
                                    shader_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                _   => return Err(VMTError::SyntaxError("Missing shader type".into()))
                            }
                        }

                        // After a shader has been loaded, anything but a block start
                        // is an error
                        State::ShaderType       => {
                            match *t {
                                Token::BlockStart   => state = State::ShaderBlockStart,
                                _ => return Err(VMTError::SyntaxError("Missing block start: {".into()))
                            }
                        }

                        // Loads a parameter type or fallback, which can occur after 
                        // a parameter value, after a Fallback or after the start of 
                        // a shader block.
                        State::ShaderBlockStart |
                        State::ShaderParamValue |
                        State::FallBlockEnd          => {
                            match *t {
                                Token::ParamType(_) => {
                                    state = State::ShaderParamType;
                                    parameter_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                Token::BlockEnd     => {
                                    state = State::ShaderBlockEnd;                                    
                                }

                                Token::BlockType(s) => {
                                    match s {
                                        "Proxies"   => (), //TODO: Proxy shit
                                        _           => {
                                            state = State::FallBlockStart;
                                            fallback_temp = Default::default();

                                            fallback_temp.f_cond = match &s[0..1] {
                                                "<" => match &s[1..2] {
                                                    "=" => FallCond::BEqual,
                                                    _   => FallCond::Below
                                                },

                                                ">" => match &s[1..2] {
                                                    "=" => FallCond::AEqual,
                                                    _   => FallCond::Above
                                                },

                                                _ => return Err(VMTError::SyntaxError("Invalid 2nd-level block name: must be a fallback or \"Proxies\"".into()))
                                            };

                                            if fallback_temp.f_cond == FallCond::BEqual ||
                                               fallback_temp.f_cond == FallCond::AEqual {
                                                fallback_temp.f_type = RSlice::from_str(&element_str[elc+2..elc+element_lens[ti]-2]);
                                            }
                                            else {
                                                fallback_temp.f_type = RSlice::from_str(&element_str[elc+1..elc+element_lens[ti]-1]);
                                            }

                                            elc += element_lens[ti];
                                            ti += 1;
                                        }
                                    }
                                }

                                _  => panic!("Oss forgot to handle all possibilites in the vmt shader loader! Please open an error on github. Also, give Oss the vmt file that crashed the program")
                            }
                            
                        }

                        State::ShaderParamType  => {
                            match *t {
                                Token::ParamValue(_) => {
                                    state = State::ShaderParamValue;

                                    parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));
                                    parameter_type = "";

                                    elc += element_lens[ti];
                                    ti += 1;
                                }
                                _   => return Err(VMTError::SyntaxError("Missing parameter value".into()))
                            }
                        }

                        State::FallBlockStart |
                        State::FallParamValue   => {
                            match *t {
                                Token::ParamType(_) => {
                                    state = State::FallParamType;

                                    parameter_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                Token::BlockStart   => {
                                    state = State::FallBlockStart
                                }

                                Token::BlockEnd => {
                                    state = State::FallBlockEnd;
                                    fallbacks.push(fallback_temp.clone());
                                }

                                _ => unreachable!()
                            }
                        }


                        State::FallParamType    => {
                            match *t {
                                Token::ParamValue(_) => {
                                    state = State::FallParamValue;

                                    fallback_temp.parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                _   => return Err(VMTError::SyntaxError("Missing paramter value in fallback".into()))
                            }
                        }

                        State::ShaderBlockEnd   => ()
                    }
                }
            }
        }

        match state {
            State::ShaderBlockEnd => (),
            _ => return Err(VMTError::SyntaxError("Unclosed block".into()))
        }
        

        Ok(Shader{s_type: RSlice::from_str(shader_type), fallbacks: fallbacks, parameters: parameters})
    }

    pub fn get_parameters(&self) -> &[Parameter] {
        &self.parameters[..]
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    // The type of parameter
    p_type: RSlice,
    // The value in the parameter
    value: RSlice
}

impl Parameter {
    fn new(p_type: &str, value: &str) -> Parameter {
        Parameter{ p_type: RSlice::from_str(p_type), value: RSlice::from_str(value)}
    }

    pub fn get_type(&self) -> &str {
        unsafe{ self.p_type.to_str() }
    }

    pub fn get_value(&self) -> &str {
        unsafe{ self.value.to_str() }
    }
}

pub type Proxy = Vec<Parameter>;

#[derive(Debug, Clone, Default)]
pub struct Fallback {
    f_cond: FallCond,
    f_type: RSlice,
    parameters: Vec<Parameter>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallCond {
    /// Above the value. Triggered on '>'
    Above,
    /// Above or equal to the value. Triggered on '>='
    AEqual,
    /// Below the value. Triggered on '<'
    Below,
    /// Below or equal to the value. Triggered on '<='
    BEqual,
    /// The default state. Should never be this state on a return.
    /// If it is, I have no idea what caused it.
    HellIfIKnow
}

impl default::Default for FallCond {
    fn default() -> FallCond {
        FallCond::HellIfIKnow
    }
}


/// A representation of a string slice with a constant pointer
/// to the location of the string and the string length. Used 
/// instead of an actual slice to get around the borrow checker.
#[derive(Clone)]
struct RSlice {
    ptr: *const u8,
    len: usize
}

impl RSlice {
    fn from_str(s: &str) -> RSlice {
        RSlice::from_raw_parts(s.as_ptr(), s.len())
    }

    #[inline(always)]
    fn from_raw_parts(ptr: *const u8, len: usize) -> RSlice {
        RSlice{ptr: ptr, len: len}
    }

    unsafe fn to_str(&self) -> &str {
        use std::slice;
        use std::str;

        let slice = slice::from_raw_parts(self.ptr, self.len);

        str::from_utf8(slice).unwrap()
    }
}

impl fmt::Display for RSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", unsafe{ self.to_str() })
    }
}

impl fmt::Debug for RSlice {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", unsafe{ self.to_str() })
    }
}

// Warning: calling functions on this default value WILL crash your program.
// Change the pointer from null to prevent this.
impl default::Default for RSlice {
    fn default() -> RSlice {
        use std::ptr;

        RSlice {
            ptr: ptr::null(),
            len: 0
        }
    }
}