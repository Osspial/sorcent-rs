use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum VTFLoadError {
    Io(io::Error),
    VTF(VTFError)
}

impl fmt::Display for VTFLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VTFLoadError::Io(ref err) => write!(f, "IO Error: {}", err),
            VTFLoadError::VTF(ref err) => write!(f, "VTF Error: {}", err)
        }
    }
}

impl error::Error for VTFLoadError {
    fn description(&self) -> &str {
        match *self {
            VTFLoadError::Io(ref err) => err.description(),
            VTFLoadError::VTF(ref err) => err.description()
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum VTFError {
    HeaderSignature,
    HeaderVersion,
    HeaderImageFormat,
    ImageSizeError,
    FileSizeError,
}

impl VTFError {
    fn __description(&self) -> &str {
        match self {
            &VTFError::HeaderSignature => "Invalid Header; Signature does not match VTF",
            &VTFError::HeaderVersion => "Invalid Header; File version does not match 7.0 - 7.5",
            &VTFError::HeaderImageFormat => "Invalid Header; Invalid image format",
            &VTFError::ImageSizeError => "Image width or height is not power of two",
            &VTFError::FileSizeError => "File too small to contain header",
        }
    }
}

impl fmt::Display for VTFError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

impl error::Error for VTFError {
    fn description(&self) -> &str {
        self.__description()
    }
}