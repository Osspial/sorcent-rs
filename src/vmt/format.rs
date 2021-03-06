use common::Token;
use vmt::error::{VMTResult, VMTError};

use std::fmt;
use std::default;
use std::marker::PhantomData;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    ShaderType,
    ShaderBlockStart,
    ShaderBlockEnd,
    ShaderParamType,
    ShaderParamValue,

    // Fallback related states
    FallBlockType,
    FallBlockStart,
    FallBlockEnd,
    FallParamType,
    FallParamValue,

    // Proxy related states
    /// State when it reads a "Proxies" block
    ProxiesDecl,
    ProxiesStart,
    ProxiesEnd,
    // For each individual proxy
    ProxyBlockType,
    ProxyBlockStart,
    ProxyBlockEnd,
    ProxyParamType,
    ProxyParamValue,

    Default
}

#[derive(Debug, Default)]
pub struct Shader<'a> {
    s_type: RSlice<'a>,
    parameters: Vec<Parameter<'a>>,
    fallbacks: Option<Vec<Fallback<'a>>>,
    proxies: Option<Vec<Proxy<'a>>>
}


impl<'a> Shader<'a> {
    /// # WARNING: HERE BE DRAGONS
    /// Hello, intrepid developer! If you've found your way into the source
    /// code of this crate, you may notice that this function is both marked
    /// unsafe and is hidden from the documentation. This is for good reason -
    /// you have no reason to use it. Seriously. A large number of invariants
    /// are unchecked, and doing so little as modifying whatever element_str
    /// is sliced from may cause the program to crash without warning. If you
    /// do use it, I take no responsibility for any lovecraftian horror you
    /// unleash upon your code.
    /// 
    /// element_lens: a vector of the lengths of each slice contained in element_str
    #[doc(hidden)]
    pub unsafe fn from_raw_parts<'s>(tokens: &Vec<Token>, element_str: &'s str, element_lens: &Vec<usize>) -> VMTResult<Shader<'a>> {
        let mut shader_type: &'s str = "";
        // Most materials don't have any fallbacks, so in most cases
        // we can avoid a heap allocation.
        let mut fallbacks: Vec<Fallback> = Vec::new();
        // Ditto for proxies
        let mut proxies: Vec<Proxy> = Vec::new();
        let mut parameters: Vec<Parameter> = Vec::with_capacity(16);
        let mut state = State::Default;

        let mut fallback_temp: Fallback = Default::default();
        let mut proxy_temp: Proxy = Default::default();

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
                        // is going to correspond to a shader. If not, then the
                        // file isn't valid.
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

                        State::ProxiesDecl      => {
                            match *t {
                                Token::BlockStart   => state = State::ProxiesStart,
                                _ => return Err(VMTError::SyntaxError("Missing proxies block start: {".into()))
                            }
                        }

                        // After a shader has been loaded, anything but a block start
                        // is an error.
                        State::ShaderType       => {
                            match *t {
                                Token::BlockStart   => state = State::ShaderBlockStart,
                                _ => return Err(VMTError::SyntaxError("Missing block start: {".into()))
                            }
                        }

                        // Ditto, for fallback
                        State::FallBlockType    => {
                            match *t {
                                Token::BlockStart   => state = State::FallBlockStart,
                                _ => return Err(VMTError::SyntaxError("Missing fallback block start: {".into()))
                            }
                        }

                        State::ProxyBlockType   => {
                            match *t {
                                Token::BlockStart   => state = State::ProxyBlockStart,
                                _ => return Err(VMTError::SyntaxError("Missing proxy block start: {".into()))
                            }
                        }

                        State::ProxiesStart |
                        State::ProxyBlockEnd    => {
                            match *t {
                                Token::BlockType(_) => {
                                    state = State::ProxyBlockType;
                                    proxy_temp = Default::default();

                                    proxy_temp.p_type = RSlice::from_str(&element_str[elc..elc+element_lens[ti]]);
                                    proxy_temp.parameters.reserve(6);

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                Token::BlockEnd     => {
                                    state = State::ProxiesEnd;
                                }

                                Token::ParamType(_) => return Err(VMTError::SyntaxError("Parameter exists in \"Proxies\" block without corresponding proxy".into())),
                                Token::BlockStart   => return Err(VMTError::SyntaxError("Block exists inside of \"Proxies\" without block tag".into())),
                                _ => unreachable!()
                            }
                        }

                        // Handles a few things - starting additional blocks inside of the
                        // shader (such as fallbacks or proxies), ending the shader block,
                        // and starting loading new shader parameters. The reason these are
                        // all handled in one group is that, once a shader block has started,
                        // a shader parameter has ended, or a fallback/proxy block has ended
                        // any of these can happen.
                        State::ShaderBlockStart |
                        State::ShaderParamValue |
                        State::FallBlockEnd     |
                        State::ProxiesEnd       => {
                            match *t {
                                Token::BlockEnd     => {
                                    state = State::ShaderBlockEnd;                                    
                                }

                                Token::BlockType(s) => {
                                    match s {
                                        "Proxies"   => {
                                            state = State::ProxiesDecl;
                                            elc += element_lens[ti];
                                            ti += 1;
                                        }

                                        _           => {
                                            state = State::FallBlockType;
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

                                Token::ParamType(_) => {
                                    state = State::ShaderParamType;
                                    parameter_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }
                                _  => panic!("Oss forgot to handle all possibilites in the vmt shader loader! Please open an error on github. Also, give Oss the vmt file that crashed the program")
                            }
                            
                        }

                        State::ProxyBlockStart |
                        State::ProxyParamValue  => {
                            match *t {
                                Token::ParamType(_) => {
                                    state = State::ProxyParamType;

                                    parameter_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                Token::BlockEnd => {
                                    state = State::ProxyBlockEnd;
                                    proxies.push(proxy_temp.clone());
                                }

                                _ => unreachable!()
                            }
                        }

                        // Once a fallback block has started, there are two options: for it
                        // to immediately end or for a parameter to start. Once a parameter has
                        // been completed, this option repeats.
                        State::FallBlockStart |
                        State::FallParamValue   => {
                            match *t {
                                Token::ParamType(_) => {
                                    state = State::FallParamType;

                                    parameter_type = &element_str[elc..elc+element_lens[ti]];

                                    elc += element_lens[ti];
                                    ti += 1;
                                }

                                Token::BlockEnd => {
                                    state = State::FallBlockEnd;
                                    fallbacks.push(fallback_temp.clone());
                                }

                                _ => unreachable!()
                            }
                        }

                        // Once a paramter has started, the differences in code are marginal
                        // so we can group most of it into a single match branch.
                        State::ShaderParamType |
                        State::FallParamType   |
                        State::ProxyParamType   => {
                            match *t {
                                Token::ParamValue(_) => {
                                    match state {
                                        State::ShaderParamType  => {
                                            parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));
                                            state = State::ShaderParamValue;
                                        }

                                        State::FallParamType    => {
                                            fallback_temp.parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));
                                            state = State::FallParamValue;
                                        }
                                        State::ProxyParamType   => {
                                            proxy_temp.parameters.push(Parameter::new(parameter_type, &element_str[elc..elc+element_lens[ti]]));
                                            state = State::ProxyParamValue;
                                        }
                                        _ => unreachable!()
                                    }

                                    parameter_type = "";

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

        let fallbacks = {
            if fallbacks.len() == 0 {
                None
            }
            else {
                Some(fallbacks)
            }
        };

        let proxies = {
            if proxies.len() == 0 {
                None
            }
            else {
                Some(proxies)
            }
        };
        

        Ok(Shader{s_type: RSlice::from_str(shader_type), parameters: parameters, fallbacks: fallbacks, proxies: proxies})
    }

    pub fn get_type(&self) -> &str {
        unsafe{ self.s_type.to_str() }
    }

    pub fn get_parameters(&self) -> &[Parameter] {
        &self.parameters[..]
    }

    pub fn get_fallbacks(&self) -> Option<&[Fallback]> {
        match self.fallbacks {
            Some(ref f) => Some(&f[..]),
            None => None
        }
    }

    pub fn get_proxies(&self) -> Option<&[Proxy]> {
        match self.proxies {
            Some(ref p) => Some(&p[..]),
            None => None
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Parameter<'a> {
    // The type of parameter
    p_type: RSlice<'a>,
    // The value in the parameter
    value: RSlice<'a>,
}

impl<'a> Parameter<'a> {
    fn new(p_type: &str, value: &str) -> Parameter<'a> {
        Parameter{ p_type: RSlice::from_str(p_type), value: RSlice::from_str(value)}
    }

    pub fn get_type(&self) -> &'a str {
        unsafe{ self.p_type.to_str() }
    }

    pub fn get_value(&self) -> &'a str {
        unsafe{ self.value.to_str() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Proxy<'a> {
    p_type: RSlice<'a>,
    parameters: Vec<Parameter<'a>>
}

impl<'a> Proxy<'a> {
    pub fn get_type(&self) -> &str {
        unsafe{ self.p_type.to_str() }
    }

    pub fn get_parameters(&self) -> &[Parameter] {
        &self.parameters[..]
    }
}

#[derive(Debug, Clone, Default)]
pub struct Fallback<'a> {
    f_cond: FallCond,
    f_type: RSlice<'a>,
    parameters: Vec<Parameter<'a>>
}

impl<'a> Fallback<'a> {
    pub fn get_condition(&self) -> FallCond {
        self.f_cond
    }

    pub fn get_type(&self) -> &str {
        unsafe{ self.f_type.to_str() }
    }

    pub fn get_parameters(&self) -> &[Parameter] {
        &self.parameters[..]
    }
}

/// A conditional statement that indicates under which
/// conditions the fallback applies (such as when the user is
/// running below DirectX 9)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
struct RSlice<'a> {
    ptr: *const u8,
    len: usize,
    life: PhantomData<&'a Shader<'a>>
}

impl<'a> RSlice<'a> {
    fn from_str(s: &str) -> RSlice<'a> {
        RSlice::from_raw_parts(s.as_ptr(), s.len())
    }

    #[inline(always)]
    fn from_raw_parts(ptr: *const u8, len: usize) -> RSlice<'a> {
        RSlice{ptr: ptr, len: len, life: PhantomData}
    }

    unsafe fn to_str(&self) -> &'a str {
        use std::slice;
        use std::str;

        let slice = slice::from_raw_parts(self.ptr, self.len);

        str::from_utf8(slice).unwrap()
    }
}

impl<'a> fmt::Display for RSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", unsafe{ self.to_str() })
    }
}

impl<'a> fmt::Debug for RSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", unsafe{ self.to_str() })
    }
}

// Warning: calling functions on this default value WILL crash your program.
// Change the pointer from null to prevent this.
impl<'a> default::Default for RSlice<'a> {
    fn default() -> RSlice<'a> {
        use std::ptr;

        RSlice {
            ptr: ptr::null(),
            len: 0,
            life: PhantomData
        }
    }
}