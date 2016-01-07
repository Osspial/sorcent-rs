use libc::{c_char, c_int, c_ushort, c_uint, c_float};

use std::io;
use std::io::Read;

use std::error;
use std::fmt;

use std::mem;

use num::FromPrimitive;


#[derive(Debug)]
pub struct Header70 {
    pub root        :RootHeader,
    pub h70         :HeaderPart70
}

impl Header for Header70 {
    fn load<R>(source: &mut R) -> Result<Header70, VTFLoadError> 
      where R: Read {
        use std::mem::transmute;

        let mut root_header_buffer: [u8; 16] = [0; 16];
        try!(source.read(&mut root_header_buffer).map_err(VTFLoadError::Io));
        let rh: RootHeaderRaw = unsafe{ transmute(root_header_buffer) };
        let root_header = unsafe{ 
            RootHeader {
                type_string: transmute(rh.type_string),
                version: transmute(rh.version),
                header_size: transmute(rh.header_size)
            } 
        };
        try!(root_header.verify().map_err(VTFLoadError::VTF));


        let mut header70_buffer: [u8; 47] = [0; 47];
        try!(source.read(&mut header70_buffer).map_err(VTFLoadError::Io));
        let h70: HeaderPart70Raw = unsafe{ transmute(header70_buffer) };
        let header70 = unsafe{ 
            HeaderPart70 {
                width: transmute(h70.width),
                height: transmute(h70.height),
                flags: transmute(h70.flags),
                frames: transmute(h70.frames),
                start_frame: transmute(h70.start_frame),
                reflectivity: transmute(h70.reflectivity),
                bump_scale: transmute(h70.bump_scale),
                image_format: try!(VTFImageFormat::from_i32(transmute(h70.image_format))
                                .ok_or(VTFLoadError::VTF(VTFError::HeaderImageFormat))),
                mip_count: h70.mip_count,
                thumbnail_format: try!(VTFImageFormat::from_i32(transmute(h70.thumbnail_format))
                                .ok_or(VTFLoadError::VTF(VTFError::HeaderImageFormat))),
                thumbnail_width: h70.thumbnail_width,
                thumbnail_height: h70.thumbnail_height
            } 
        };
        try!(header70.verify().map_err(VTFLoadError::VTF));

        Ok(Header70{ root: root_header, h70: header70 })
    }
}

pub type Header71 = Header70;


#[derive(Debug)]
pub struct Header72 {
    pub root        :RootHeader,
    pub h70         :HeaderPart70,
    pub h72         :HeaderPart72
}


impl Header for Header72 {
    fn load<R>(source: &mut R) -> Result<Header72, VTFLoadError> 
      where R: Read {
        use std::mem::transmute;

        let header70 = try!(Header70::load(source));


        let mut header72_buffer: [u8; 2] = [0; 2];
        try!(source.read(&mut header72_buffer).map_err(VTFLoadError::Io));
        let h72: HeaderPart72Raw = unsafe{ transmute(header72_buffer) };
        let header72 = unsafe {
            HeaderPart72 {
                depth: transmute(h72.depth)
            }
        };


        Ok(Header72{ root: header70.root, h70: header70.h70, h72: header72 })
    }
}



#[derive(Debug)]
pub struct Header73 {
    pub root        :RootHeader,
    pub h70         :HeaderPart70,
    pub h72         :HeaderPart72,
    pub h73         :HeaderPart73
}

impl Header for Header73 {
    fn load<R>(source: &mut R) -> Result<Header73, VTFLoadError> 
      where R: Read {
        use std::mem::transmute;

        let header72 = try!(Header72::load(source));


        let mut header73_buffer: [u8; 7] = [0; 7];
        try!(source.read(&mut header73_buffer).map_err(VTFLoadError::Io));
        let h73: HeaderPart73Raw = unsafe{ transmute(header73_buffer) };
        let header73 = unsafe {
            HeaderPart73 {
                resource_count: transmute(h73.resource_count)
            }
        };

        Ok(Header73{ root: header72.root, h70: header72.h70, h72: header72.h72, h73: header73 })
    }
}

pub type Header74 = Header73;
pub type Header75 = Header74;

pub trait Header 
    where Self: Sized {

    fn load<R>(source: &mut R) -> Result<Self, VTFLoadError> where R: Read;
}



#[derive(Debug)]
pub struct RootHeader {
    pub type_string         :[c_char; 4],
    pub version             :[c_int; 2],
    pub header_size         :c_int
}

impl RootHeader {
    fn verify(&self) -> Result<(), VTFError> {
        let magic_number: [i8; 4] = unsafe{ mem::transmute(*b"VTF\0") };
        if self.type_string != magic_number {
            Err(VTFError::HeaderSignature)
        }
        else if self.version[0] != 7 || match self.version[1] {1 ... 5 => false, _ => true} {
            Err(VTFError::HeaderVersion)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct HeaderPart70 {
    pub width               :c_ushort,
    pub height              :c_ushort,
    pub flags               :c_uint,
    pub frames              :c_ushort,
    pub start_frame         :c_ushort, 
    pub reflectivity        :[c_float; 3],
    pub bump_scale          :c_float,
    pub image_format        :VTFImageFormat,
    pub mip_count           :u8,
    pub thumbnail_format    :VTFImageFormat,
    pub thumbnail_width     :u8,
    pub thumbnail_height    :u8
}

impl HeaderPart70 {
    fn verify(&self) -> Result<(), VTFError> {
        //Checks to see if width or hight is not a power of two
        if !(self.width.is_power_of_two() || self.height.is_power_of_two()) {
            Err(VTFError::ImageSizeError)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct HeaderPart72 {
    pub depth           :c_ushort
}

#[derive(Debug)]
pub struct HeaderPart73 {
    pub resource_count  :c_uint
}


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

struct Resource {
    rtype       :VTFResourceType,
    flags       :u8,
    data        :u32
}

impl Resource {
    fn load<R>(/*source: &mut R*/) /*-> Result<Resource, VTFLoadError>*/ {

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







///RootHeader as arrays of unsigned integers to assist in loading
///Size: 16
#[derive(Default, Debug)]
#[repr(C)]
struct RootHeaderRaw {
    type_string         :[u8; 4],
    version             :[u8; 8],
    header_size         :[u8; 4]
}


///Header70 as arrays of unsigned integers to assist in loading
///Size: 47
#[derive(Default, Debug)]
#[repr(C)]
struct HeaderPart70Raw {
    width               :[u8; 2],
    height              :[u8; 2],
    flags               :[u8; 4],
    frames              :[u8; 2],
    start_frame         :[u8; 2],
    padding_0           :[u8; 4],
    reflectivity        :[u8; 12],
    padding_1           :[u8; 4],
    bump_scale          :[u8; 4],
    image_format        :[u8; 4],
    mip_count           :u8,
    thumbnail_format    :[u8; 4],
    thumbnail_width     :u8,
    thumbnail_height    :u8
}

///Header72 as arrays of unsigned integers to assist in loading
///Size: 2
#[derive(Default, Debug)]
#[repr(C)]
struct HeaderPart72Raw {
    depth               :[u8; 2]
}

///Header73 as arrays of unsigned integers to assist in loading
///Size: 7
#[derive(Default, Debug)]
#[repr(C)]
struct HeaderPart73Raw {
    padding             :[u8; 3],
    resource_count      :[u8; 4]
}


#[derive(Default, Debug)]
#[repr(C)]
struct ResourceRaw {
    rtype               :[u8; 3],
    flags               :u8,
    data                :[u8; 4]
}




/*
//Functions used to compute the resource IDs.
//Commented out as they are hard-coded into the enum as rust
//doesn't support compile-time function evaluation

fn make_vtf_rsrc_id(a: u8, b: u8, c: u8) -> i32 {
    let (a, b, c) = (a as i32, b as i32, c as i32);

    (a | b << 8 | c << 16) as i32
}

fn make_vtf_rsrc_idf(a: u8, b: u8, c: u8, f: u8) -> i32 {
    let (a, b, c, f) = (a as i32, b as i32, c as i32, f as i32);

    (a | b << 8 | c << 16 | f << 24) as i32
}

*/

const RSRC_NO_DATA_CHUNK: u8 = 0x02;


#[derive(Debug)]
#[repr(u32)]
pub enum VTFResourceType {
    LegacyLowResImage       = 0x01, //make_vtf_rsrc_id(0x01, 0, 0)
    LegacyImage             = 0x30, //make_vtf_rsrc_id(0x30, 0, 0)
    Sheet                   = 0x10, //make_vtf_rsrc_id(0x10, 0, 0)
    Crc                     = 0x435243, //make_vtf_rsrc_idf('C', 'R', 'C', RSRC_NO_DATA_CHUNK)
    TextureLODSettings      = 0x444f4c, //make_vtf_rsrc_idf('L', 'O', 'D', RSRC_NO_DATA_CHUNK)
    TextureSettingsEx       = 0x4f5354, //make_vtf_rsrc_idf('T', 'S', 'O', RSRC_NO_DATA_CHUNK)
    KeyValueData            = 0x44564b, //make_vtf_rsrc_id('K', 'V', 'D')
    MaxDictionaryEntries    = 32, //32
}

impl FromPrimitive for VTFResourceType {
    fn from_i64(n: i64) -> Option<VTFResourceType> {
        match n {
            0x01        => Some(VTFResourceType::LegacyLowResImage),
            0x30        => Some(VTFResourceType::LegacyImage),
            0x10        => Some(VTFResourceType::Sheet),
            0x435243    => Some(VTFResourceType::Crc),
            0x444f4c    => Some(VTFResourceType::TextureLODSettings),
            0x4f5354    => Some(VTFResourceType::TextureSettingsEx),
            0x44564b    => Some(VTFResourceType::KeyValueData),
            32          => Some(VTFResourceType::MaxDictionaryEntries),
            _           => None
        }
    }

    fn from_u64(n: u64) -> Option<VTFResourceType> {
        VTFResourceType::from_i64(n as i64)
    }
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

impl FromPrimitive for VTFImageFormat {
    fn from_i64(n: i64) -> Option<VTFImageFormat> {
        match n {
            0  => Some(VTFImageFormat::IMAGE_FORMAT_RGBA8888),
            1  => Some(VTFImageFormat::IMAGE_FORMAT_ABGR8888),
            2  => Some(VTFImageFormat::IMAGE_FORMAT_RGB888),
            3  => Some(VTFImageFormat::IMAGE_FORMAT_BGR888),
            4  => Some(VTFImageFormat::IMAGE_FORMAT_RGB565),
            5  => Some(VTFImageFormat::IMAGE_FORMAT_I8),
            6  => Some(VTFImageFormat::IMAGE_FORMAT_IA88),
            7  => Some(VTFImageFormat::IMAGE_FORMAT_P8),
            8  => Some(VTFImageFormat::IMAGE_FORMAT_A8),
            9  => Some(VTFImageFormat::IMAGE_FORMAT_RGB888_BLUESCREEN),
            10 => Some(VTFImageFormat::IMAGE_FORMAT_BGR888_BLUESCREEN),
            11 => Some(VTFImageFormat::IMAGE_FORMAT_ARGB8888),
            12 => Some(VTFImageFormat::IMAGE_FORMAT_BGRA8888),
            13 => Some(VTFImageFormat::IMAGE_FORMAT_DXT1),
            14 => Some(VTFImageFormat::IMAGE_FORMAT_DXT3),
            15 => Some(VTFImageFormat::IMAGE_FORMAT_DXT5),
            16 => Some(VTFImageFormat::IMAGE_FORMAT_BGRX8888),
            17 => Some(VTFImageFormat::IMAGE_FORMAT_BGR565),
            18 => Some(VTFImageFormat::IMAGE_FORMAT_BGRX5551),
            19 => Some(VTFImageFormat::IMAGE_FORMAT_BGRA4444),
            20 => Some(VTFImageFormat::IMAGE_FORMAT_DXT1_ONEBITALPHA),
            21 => Some(VTFImageFormat::IMAGE_FORMAT_BGRA5551),
            22 => Some(VTFImageFormat::IMAGE_FORMAT_UV88),
            23 => Some(VTFImageFormat::IMAGE_FORMAT_UVWQ8888),
            24 => Some(VTFImageFormat::IMAGE_FORMAT_RGBA16161616F),
            25 => Some(VTFImageFormat::IMAGE_FORMAT_RGBA16161616),
            26 => Some(VTFImageFormat::IMAGE_FORMAT_UVLX8888),
            27 => Some(VTFImageFormat::IMAGE_FORMAT_R32F),
            28 => Some(VTFImageFormat::IMAGE_FORMAT_RGB323232F),
            29 => Some(VTFImageFormat::IMAGE_FORMAT_RGBA32323232F),
            30 => Some(VTFImageFormat::IMAGE_FORMAT_NV_DST16),
            31 => Some(VTFImageFormat::IMAGE_FORMAT_NV_DST24),                  
            32 => Some(VTFImageFormat::IMAGE_FORMAT_NV_INTZ),
            33 => Some(VTFImageFormat::IMAGE_FORMAT_NV_RAWZ),
            34 => Some(VTFImageFormat::IMAGE_FORMAT_ATI_DST16),
            35 => Some(VTFImageFormat::IMAGE_FORMAT_ATI_DST24),
            36 => Some(VTFImageFormat::IMAGE_FORMAT_NV_NULL),
            37 => Some(VTFImageFormat::IMAGE_FORMAT_ATI2N),                     
            38 => Some(VTFImageFormat::IMAGE_FORMAT_ATI1N),
            39 => Some(VTFImageFormat::IMAGE_FORMAT_COUNT),
            -1 => Some(VTFImageFormat::IMAGE_FORMAT_NONE),
            _ => None
        }
    }

    fn from_u64(n: u64) -> Option<VTFImageFormat> {
        match n {
            0 ... 40 => VTFImageFormat::from_i64(n as i64),
            _ => None
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(u32)]
pub enum VTFFlags {
    TEXTUREFLAGS_POINTSAMPLE        = 0x00000001,
    TEXTUREFLAGS_TRILINEAR          = 0x00000002,
    TEXTUREFLAGS_CLAMPS             = 0x00000004,
    TEXTUREFLAGS_CLAMPT             = 0x00000008,
    TEXTUREFLAGS_ANISOTROPIC        = 0x00000010,
    TEXTUREFLAGS_HINT_DXT5          = 0x00000020,
    TEXTUREFLAGS_PWL_CORRECTED      = 0x00000040,
    TEXTUREFLAGS_NORMAL             = 0x00000080,
    TEXTUREFLAGS_NOMIP              = 0x00000100,
    TEXTUREFLAGS_NOLOD              = 0x00000200,
    TEXTUREFLAGS_ALL_MIPS           = 0x00000400,
    TEXTUREFLAGS_PROCEDURAL         = 0x00000800,

    // These are automatically generated by vtex from the texture data.
    TEXTUREFLAGS_ONEBITALPHA        = 0x00001000,
    TEXTUREFLAGS_EIGHTBITALPHA      = 0x00002000,

    // Newer flags from the *.txt config file
    TEXTUREFLAGS_ENVMAP             = 0x00004000,
    TEXTUREFLAGS_RENDERTARGET       = 0x00008000,
    TEXTUREFLAGS_DEPTHRENDERTARGET  = 0x00010000,
    TEXTUREFLAGS_NODEBUGOVERRIDE    = 0x00020000,
    TEXTUREFLAGS_SINGLECOPY         = 0x00040000,
    TEXTUREFLAGS_PRE_SRGB           = 0x00080000,
 
    TEXTUREFLAGS_UNUSED0            = 0x00100000,
    TEXTUREFLAGS_UNUSED1            = 0x00200000,
    TEXTUREFLAGS_UNUSED2            = 0x00400000,
 
    TEXTUREFLAGS_NODEPTHBUFFER      = 0x00800000,
 
    TEXTUREFLAGS_UNUSED3           = 0x01000000,
 
    TEXTUREFLAGS_CLAMPU             = 0x02000000,
    TEXTUREFLAGS_VERTEXTEXTURE      = 0x04000000,
    TEXTUREFLAGS_SSBUMP             = 0x08000000,           
 
    TEXTUREFLAGS_UNUSED4            = 0x10000000,
 
    TEXTUREFLAGS_BORDER             = 0x20000000,
 
    TEXTUREFLAGS_UNUSED_40000000    = 0x40000000,
    TEXTUREFLAGS_UNUSED_80000000    = 0x80000000,
}