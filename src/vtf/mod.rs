#[allow(dead_code)]
mod format;
mod error;
#[allow(dead_code)]
pub mod dxt;

use std::fs::File;
use std::io::Read;
use std::mem;

use self::format::{Data, HeaderRoot, Header70, Header72, Header73, Resource, HeaderVersion};
use self::error::{VTFLoadError, VTFError};
use self::dxt::Dxt1Raw;

#[derive(Debug)]
pub struct VTFFile {
    header: HeaderVersion,
    resources: Option<Box<[Resource]>>,
    image: Option<Dxt1Raw>
}

impl VTFFile {
    pub fn open(file: &mut File) -> Result<VTFFile, VTFLoadError> {
        let metadata = file.metadata().unwrap();        

        //Size check
        if metadata.len() < mem::size_of::<format::Header70>() as u64 {
            return Err(VTFLoadError::VTF(VTFError::FileSize));
        }

        let header: HeaderVersion;
        let header_root = try!(HeaderRoot::load(&mut *file));


        if header_root.version == [7, 3] || header_root.version == [7, 4] || header_root.version == [7, 5] {
            header = HeaderVersion::H73(
                                header_root, 
                                try!(Header70::load(&mut *file)),
                                try!(Header72::load(&mut *file)),
                                try!(Header73::load(&mut *file)));

            let resource_count: usize;

            
            match &header {
                &HeaderVersion::H73(_, _, _, ref h) => resource_count = h.resource_count as usize,
                _ => unreachable!()
            }
            
            //Throw away the header padding into an empty buffer
            let mut padding: [u8; 8] = [0; 8];
            try!(file.read(&mut padding).map_err(VTFLoadError::Io));
            
            //Create a vector with a capacity of the header's listed resource count
            let mut resources: Vec<Resource> = Vec::with_capacity(resource_count as usize);
            let mut ri = 0; //Resource index for loop
            while ri < resource_count {
                resources.push(try!(Resource::load(&mut *file)));
                ri += 1;
            }

            let image = Dxt1Raw::load(&mut *file, 16*16).unwrap();

            Ok(VTFFile {header: header, resources: Some(resources.into_boxed_slice()), image: Some(image)})

        } else if header_root.version == [7, 2] {
            header = HeaderVersion::H72(
                                header_root, 
                                try!(Header70::load(&mut *file)),
                                try!(Header72::load(&mut *file)));
            Ok(VTFFile {header: header, resources: None, image: None})

        } else if header_root.version == [7, 1] || header_root.version == [7, 0] {
            header = HeaderVersion::H70(
                                header_root, 
                                try!(Header70::load(&mut *file)));
            Ok(VTFFile {header: header, resources: None, image: None})

        } else {
            Err(VTFLoadError::VTF(VTFError::HeaderVersion))
        }
    }
}