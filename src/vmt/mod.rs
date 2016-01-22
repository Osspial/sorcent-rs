#[allow(dead_code)]
pub mod format;
#[allow(dead_code)]
pub mod types;
pub mod error;

use self::format::{Shader};
use self::error::VMTLoadError;
use std::fs::File;
use std::io::{BufReader};

pub struct VMTFile {
    shader: Shader
}

/*
impl VMTFile {
    pub fn open(file: &mut File) -> Result<VMTFile, VMTLoadError> {
        let mut buf_read = BufReader::new(file);

        let shader = Shader::load(&mut buf_read).unwrap();

        Ok(VMTFile{ shader: shader })
    }
}
*/