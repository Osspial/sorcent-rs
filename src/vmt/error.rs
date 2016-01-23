use std::error;
use std::fmt;
use std::io;

pub type VMTLoadResult<T> = Result<T, VMTLoadError>;
pub type VMTResult<T> = Result<T, VMTError>;

#[derive(Debug)]
pub enum VMTLoadError {
    Io(io::Error),
    VMT(VMTError)
}

impl fmt::Display for VMTLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VMTLoadError::Io(ref err) => write!(f, "IO Error: {}", err),
            VMTLoadError::VMT(ref err) => write!(f, "VMT Error: {}", err)
        }
    }
}

impl error::Error for VMTLoadError {
    fn description(&self) -> &str {
        match *self {
            VMTLoadError::Io(ref err) => err.description(),
            VMTLoadError::VMT(ref err) => err.description()
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VMTError {
    UnclosedBlock,
    InvalidToken, // TODO: Add additional description to InvalidToken
    SyntaxError(String),
    UnknownShader(String),
    UnknownParameter(String),
}

impl VMTError {
    fn description(&self) -> &str {
        match self {
            &VMTError::UnclosedBlock        => "Unclosed Block; missing \"}\"",
            &VMTError::InvalidToken         => "Invalid token",
            &VMTError::SyntaxError(_)       => "Syntax Error",
            &VMTError::UnknownShader(_)     => "Unknown Shader found",
            &VMTError::UnknownParameter(_)  => "Unknown Parameter found"
        }
    }
}

impl fmt::Display for VMTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match self {
            &VMTError::UnknownShader(ref s)     => format!("Unknown Shader: {}", s).fmt(f),
            &VMTError::UnknownParameter(ref s)  => format!("Unknown Parameter: {}", s).fmt(f),
            &VMTError::SyntaxError(ref s)       => format!("Syntax Error: {}", s).fmt(f),
            _                                   => self.description().fmt(f)
        }
    }
}

impl error::Error for VMTError {
    fn description(&self) -> &str {
        self.description()
    }
}