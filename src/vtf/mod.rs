#[allow(dead_code)]
mod format;
mod error;
#[allow(dead_code)]
pub mod dxt;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem;

use self::format::{Data, HeaderRoot, Header70, Header72, Header73, Resource, ResourceID, HeaderVersion};
use self::error::{VTFLoadError, VTFError};
use self::dxt::Dxt1Raw;

#[derive(Debug)]
pub struct VTFFile {
    pub header: HeaderVersion,
    pub resources: Option<Box<[Resource]>>,
    pub thumb: Option<Dxt1Raw>,
    pub mips: Option<Vec<Dxt1Raw>>,
    pub image: Option<Dxt1Raw>
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

            
            
            let thumb: Dxt1Raw;
            let mut mips: Vec<Dxt1Raw>;
            let image: Dxt1Raw;
            //Create a vector with a capacity of the header's listed resource count
            let mut resources: Vec<Resource>;
            {
                let header_root = header.get_root();
                let header70 = header.get_h70();
                let header73 = header.get_h73().unwrap();

                let resource_count = header73.resource_count as usize;
                
                file.seek(SeekFrom::Start((header_root.header_size - header73.resource_count as i32*8) as u64)).unwrap();
                
                resources = Vec::with_capacity(resource_count as usize);

                let mut thumb_ri: usize = 0; //Index of thumbnail resource
                let mut image_ri: usize = 0; //Index of image resource
                let mut ri = 0; //Resource index for loop
                while ri < resource_count {
                    resources.push(try!(Resource::load(&mut *file)));

                    // Figure out if the loaded resource is a thumbnail or image resource, and if it is
                    // store the index
                    match &resources[ri].id {
                        &ResourceID::LegacyLowResImage => thumb_ri = ri,
                        &ResourceID::LegacyImage => image_ri = ri,
                        _ => ()
                    }
                    ri += 1;
                }
                let (thumb_ri, image_ri) = (thumb_ri, image_ri); //Remove mutability from indices

                file.seek(SeekFrom::Start(resources[thumb_ri].data as u64)).unwrap();
                thumb = try!(Dxt1Raw::load(&mut *file, header70.thumbnail_width as u16, header70.thumbnail_height as u16).map_err(VTFLoadError::Io));


                file.seek(SeekFrom::Start(resources[image_ri].data as u64)).unwrap();

                
                mips = Vec::with_capacity((header70.mip_count - 1) as usize);
                // Load mipmap data
                {
                    let mut mip_width = header70.width / 2u16.pow(header70.mip_count as u32 - 1);
                    let mut mip_height = header70.height / 2u16.pow(header70.mip_count as u32 - 1);
                    while mip_width < header70.width {
                        mips.push(Dxt1Raw::load(&mut *file, mip_width, mip_height).unwrap());
                        mip_width *= 2;
                        mip_height *= 2;
                    }
                }
                

                image = try!(Dxt1Raw::load(&mut *file, header70.width, header70.height).map_err(VTFLoadError::Io));
            }

            Ok(VTFFile {header: header, resources: Some(resources.into_boxed_slice()), thumb: Some(thumb), mips: Some(mips), image: Some(image)})

        } else if header_root.version == [7, 2] {
            header = HeaderVersion::H72(
                                header_root, 
                                try!(Header70::load(&mut *file)),
                                try!(Header72::load(&mut *file)));
            Ok(VTFFile {header: header, resources: None, thumb: None, mips: None, image: None})

        } else if header_root.version == [7, 1] || header_root.version == [7, 0] {
            header = HeaderVersion::H70(
                                header_root, 
                                try!(Header70::load(&mut *file)));
            Ok(VTFFile {header: header, resources: None, thumb: None, mips: None, image: None})

        } else {
            Err(VTFLoadError::VTF(VTFError::HeaderVersion))
        }
    }
}