use std::error;
use std::fmt;
use std::io;

pub struct Shader {
    name: String,
    parameters: Vec<Parameter>,
    proxies: Vec<Proxy>
}

pub struct Proxy {
    name: String,
    parameters: Vec<Parameter>
}

pub struct Parameter {
    name: String,
    value: String,
}

pub trait VMTData 
    where Self: Sized {
        
    fn load<R>(source: R) -> Result<Self, VMTLoadError>;
}


#[derive(Debug)]
pub enum VMTLoadError {
    Io(io::Error),
    VMT(VMTError)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMTError {
    UnclosedBlock
}

impl VMTError {
    fn description(&self) -> &str {
        match self {
            &VMTError::UnclosedBlock => "Unclosed Block; missing \"}\""
        }
    }
}

impl fmt::Display for VMTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl error::Error for VMTError {
    fn description(&self) -> &str {
        self.description()
    }
}