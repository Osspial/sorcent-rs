#[allow(dead_code)]
mod format;
mod error;

use std::fs::File;
use std::io::Read;
use std::mem;

use self::format::{Data, HeaderRoot, Header70, Header72, Header73, Resource, HeaderVersion};
use self::error::{VTFLoadError, VTFError};

#[derive(Debug)]
pub struct VTFFile {
    header: HeaderVersion,
    //resources: &'v [Resource]
}

impl VTFFile {
    pub fn open(file: &mut File) -> Result<VTFFile, VTFLoadError> {
        let metadata = file.metadata().unwrap();        

        //Size check
        if metadata.len() < mem::size_of::<format::Header70>() as u64 {
            return Err(VTFLoadError::VTF(VTFError::FileSizeError));
        }

        let mut header: HeaderVersion;
        let header_root = try!(HeaderRoot::load(&mut *file));

        if header_root.version == [7, 3] || header_root.version == [7, 4] || header_root.version == [7, 5] {
            header = HeaderVersion::H73(
                                header_root, 
                                try!(Header70::load(&mut *file)),
                                try!(Header72::load(&mut *file)),
                                try!(Header73::load(&mut *file)));
        } else if header_root.version == [7, 2] {
            header = HeaderVersion::H72(
                                header_root, 
                                try!(Header70::load(&mut *file)),
                                try!(Header72::load(&mut *file)));
        } else if header_root.version == [7, 1] || header_root.version == [7, 0] {
            header = HeaderVersion::H70(
                                header_root, 
                                try!(Header70::load(&mut *file)));
        } else {
            header = HeaderVersion::Empty
        }


        Ok(VTFFile {header: header})
    }
}