use libc::{c_char, c_int, c_ushort, c_uint, c_float};
use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct RootHeader {
    pub type_string: [c_char; 4],
    pub version: [c_int; 2],
    pub header_size: c_int
}

#[derive(Debug)]
pub struct Header70Part {
    pub width: c_ushort,
    pub height: c_ushort,
    pub flags: c_uint,
    pub frames: c_ushort,
    pub start_frame: c_ushort, 
    pub reflectivity: [c_float; 3],
    pub bump_scale: c_float,
    pub image_format: VTFImageFormat,
    pub mip_count: u8,
    pub thumbnail_format: VTFImageFormat,
    pub thumbnail_width: u8,
    pub thumbnail_height: u8
}

#[derive(Debug)]
pub struct Header70 {
    pub root: RootHeader,
    pub h70: Header70Part
}

impl Header70 {
    pub fn open<R>(source: &mut R) -> Result<Header70, io::Error> where R: Read {
        use std::mem;

        let mut root_header_buffer: [u8; 16] = [0; 16];
        try!(source.read(&mut root_header_buffer));
        let root_header_raw: RootHeaderRaw = unsafe{ mem::transmute(root_header_buffer) };


        let mut header70_buffer: [u8; 47] = [0; 47];
        try!(source.read(&mut header70_buffer));
        let header70_raw: Header70PartRaw = unsafe{ mem::transmute(header70_buffer) };

        Ok(unsafe{ Header70::from_raw_parts(root_header_raw, header70_raw) })
    }

    unsafe fn from_raw_parts(rh: RootHeaderRaw, h70: Header70PartRaw) -> Header70 {
        use std::mem::transmute;

        Header70 {
            root: RootHeader {
                type_string: transmute(rh.type_string),
                version: transmute(rh.version),
                header_size: transmute(rh.header_size)
            },
            h70: Header70Part {
                width: transmute(h70.width),
                height: transmute(h70.height),
                flags: transmute(h70.flags),
                frames: transmute(h70.frames),
                start_frame: transmute(h70.start_frame),
                reflectivity: transmute(h70.reflectivity),
                bump_scale: transmute(h70.bump_scale),
                image_format: transmute(h70.image_format),
                mip_count: h70.mip_count,
                thumbnail_format: transmute(h70.thumbnail_format),
                thumbnail_width: h70.thumbnail_width,
                thumbnail_height: h70.thumbnail_height
            }
        }
    }
}

pub type Header71 = Header70;






///RootHeader as arrays of unsigned integers to assist in loading
///Size: 16
#[derive(Default, Debug)]
#[repr(C)]
struct RootHeaderRaw {
    type_string: [u8; 4],
    version: [u8; 8],
    header_size: [u8; 4]
}


///Header70 as arrays of unsigned integers to assist in loading
///Size: 47
#[derive(Default, Debug)]
#[repr(C)]
struct Header70PartRaw {
    width: [u8; 2],
    height: [u8; 2],
    flags: [u8; 4],
    frames: [u8; 2],
    start_frame: [u8; 2],
    padding_0: [u8; 4],
    reflectivity: [u8; 12],
    padding_1: [u8; 4],
    bump_scale: [u8; 4],
    image_format: [u8; 4],
    mip_count: u8,
    thumbnail_format: [u8; 4],
    thumbnail_width: u8,
    thumbnail_height: u8
}


#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(i32)]
pub enum VTFImageFormat {
    IMAGE_FORMAT_RGBA8888 = 0,
    IMAGE_FORMAT_ABGR8888,
    IMAGE_FORMAT_RGB888,
    IMAGE_FORMAT_BGR888,
    IMAGE_FORMAT_RGB565,
    IMAGE_FORMAT_I8,
    IMAGE_FORMAT_IA88,
    IMAGE_FORMAT_P8,
    IMAGE_FORMAT_A8,
    IMAGE_FORMAT_RGB888_BLUESCREEN,
    IMAGE_FORMAT_BGR888_BLUESCREEN,
    IMAGE_FORMAT_ARGB8888,
    IMAGE_FORMAT_BGRA8888,
    IMAGE_FORMAT_DXT1,
    IMAGE_FORMAT_DXT3,
    IMAGE_FORMAT_DXT5,
    IMAGE_FORMAT_BGRX8888,
    IMAGE_FORMAT_BGR565,
    IMAGE_FORMAT_BGRX5551,
    IMAGE_FORMAT_BGRA4444,
    IMAGE_FORMAT_DXT1_ONEBITALPHA,
    IMAGE_FORMAT_BGRA5551,
    IMAGE_FORMAT_UV88,
    IMAGE_FORMAT_UVWQ8888,
    IMAGE_FORMAT_RGBA16161616F,
    IMAGE_FORMAT_RGBA16161616,
    IMAGE_FORMAT_UVLX8888,
    IMAGE_FORMAT_R32F,
    IMAGE_FORMAT_RGB323232F,
    IMAGE_FORMAT_RGBA32323232F,
    IMAGE_FORMAT_NV_DST16,
    IMAGE_FORMAT_NV_DST24,                  
    IMAGE_FORMAT_NV_INTZ,
    IMAGE_FORMAT_NV_RAWZ,
    IMAGE_FORMAT_ATI_DST16,
    IMAGE_FORMAT_ATI_DST24,
    IMAGE_FORMAT_NV_NULL,
    IMAGE_FORMAT_ATI2N,                     
    IMAGE_FORMAT_ATI1N,
    IMAGE_FORMAT_COUNT,
    IMAGE_FORMAT_NONE = -1
}

impl Default for VTFImageFormat {
    fn default() -> Self {
        VTFImageFormat::IMAGE_FORMAT_NONE
    }
}