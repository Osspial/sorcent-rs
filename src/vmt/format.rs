use vmt::lexer::Token;
use vmt::error::{VMTResult, VMTError};

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
pub struct Shader<'s> {
    pub s_type: &'s str,
    pub parameters: Vec<Parameter<'s>>,
    //pub proxies: Vec<Proxy<'s>>
}


impl<'s> Shader<'s> {
    /// element_lens: a vector of the lengths of each slice contained in element_str
    pub fn from_raw_parts(tokens: &Vec<Token>, element_str: &'s str, element_lens: &Vec<usize>) -> VMTResult<Shader<'s>> {
        let empty_slice = &element_str[0..0];

        let mut shader_type: &'s str = empty_slice;
        let mut parameters: Vec<Parameter<'s>> = Vec::with_capacity(16);
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

                        State::ShaderType       => {
                            match *t {
                                Token::BlockStart   => state = State::ShaderBlockStart,
                                _ => return Err(VMTError::SyntaxError("Missing block start: {".into()))
                            }
                        }

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
        

        Ok(Shader{s_type: shader_type, parameters: parameters})
    }
}

#[derive(Debug)]
pub struct Parameter<'s> {
    pub p_type: &'s str,
    pub value: &'s str
}

impl<'s> Parameter<'s> {
    pub fn new(p_type: &'s str, value: &'s str) -> Parameter<'s> {
        Parameter{p_type: p_type, value: value}
    }
}

pub struct Proxy<'s> {
    pub r_type: &'s str,
    pub parameters: Vec<Parameter<'s>>
}