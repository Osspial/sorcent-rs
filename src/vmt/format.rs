use vmt::lexer::Token;
use vmt::error::{VMTResult, VMTError};

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    ShaderType,
    ShaderBlockStart,
    ShaderBlockEnd,

    ShaderParamType,
    ShaderParamValue,

    Default
}

#[derive(Debug)]
pub struct Shader {
    s_type: RSlice,
    parameters: Vec<Parameter>,
    //pub proxies: Vec<Proxy<'s>>
}


impl Shader {
    /// element_lens: a vector of the lengths of each slice contained in element_str
    pub fn from_raw_parts<'s>(tokens: &Vec<Token>, element_str: &'s str, element_lens: &Vec<usize>) -> VMTResult<Shader> {
        let empty_slice = &element_str[0..0];

        let mut shader_type: &'s str = empty_slice;
        let mut parameters: Vec<Parameter> = Vec::with_capacity(16);
        let mut state = State::Default;

        // What number token we're on
        let mut ti = 0;
        // Where in the element string we are
        let mut elc = 0;

        // Temporary storage locations for parameter types
        let mut parameter_type: &'s str = empty_slice;
        
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

                        // Loads a parameter type, which can occur after a parameter
                        // value or after the start of a block.
                        State::ShaderBlockStart |
                        State::ShaderParamValue => {
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

                                _  => panic!("Oss forgot to handle all possibilites in the vmt shader loader! Please open an error on github")
                            }
                            
                        }

                        State::ShaderParamType  => {
                            match *t {
                                Token::ParamValue(_) => {
                                    if parameter_type == empty_slice {
                                        panic!("Somehow, a parameter value exists without a parameter type. This should never have happened. What the hell did you do?!‽?!");
                                    }
                                    state = State::ShaderParamValue;

                                    parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));
                                    parameter_type = empty_slice;

                                    elc += element_lens[ti];
                                    ti += 1;
                                }
                                _   => return Err(VMTError::SyntaxError("Missing parameter value".into()))
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
        

        Ok(Shader{s_type: RSlice::from_str(shader_type), parameters: parameters})
    }

    pub fn get_parameters(&self) -> &[Parameter] {
        &self.parameters[..]
    }
}

#[derive(Debug)]
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

/// A representation of a string slice with a constant pointer
/// to the location of the string and the string length. Used 
/// instead of an actual slice to get around the borrow checker.
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

pub type Proxy = Vec<Parameter>;