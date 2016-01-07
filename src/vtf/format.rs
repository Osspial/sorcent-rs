use libc::{c_char, c_int, c_ushort, c_uint, c_float};
use std::io::Read;
use std::mem;
use num::FromPrimitive;
use super::error::*;


#[derive(Debug)]
pub enum HeaderVersion {
    H70(HeaderRoot, Header70),
    H72(HeaderRoot, Header70, Header72),
    H73(HeaderRoot, Header70, Header72, Header73),
}

#[derive(Debug)]
pub struct HeaderRoot {
    pub type_string         :[c_char; 4],
    pub version             :[c_int; 2],
    pub header_size         :c_int
}

impl HeaderRoot {
    fn verify(&self) -> Result<(), VTFError> {
        let magic_number: [i8; 4] = unsafe{ mem::transmute(*b"VTF\0") };
        if self.type_string != magic_number {
            Err(VTFError::HeaderSignature)
        } else if self.version[0] != 7 || match self.version[1] {1 ... 5 => false, _ => true} {
            Err(VTFError::HeaderVersion)
        } else {
            Ok(())
        }
    }
}

impl Data for HeaderRoot {
    fn load<R>(source: &mut R) -> Result<HeaderRoot, VTFLoadError> where R: Read {
        use std::mem::transmute;

        let mut root_header_buffer: [u8; 16] = [0; 16];
        try!(source.read(&mut root_header_buffer).map_err(VTFLoadError::Io));
        let rh: HeaderRootRaw = unsafe{ transmute(root_header_buffer) };
        let root_header = unsafe{ 
            HeaderRoot {
                type_string: transmute(rh.type_string),
                version: transmute(rh.version),
                header_size: transmute(rh.header_size)
            } 
        };
        try!(root_header.verify().map_err(VTFLoadError::VTF));

        Ok(root_header)
    }
}

#[derive(Debug)]
pub struct Header70 {
    pub width               :c_ushort,
    pub height              :c_ushort,
    pub flags               :c_uint,
    pub frames              :c_ushort,
    pub start_frame         :c_ushort, 
    pub reflectivity        :[c_float; 3],
    pub bump_scale          :c_float,
    pub image_format        :ImageFormat,
    pub mip_count           :u8,
    pub thumbnail_format    :ImageFormat,
    pub thumbnail_width     :u8,
    pub thumbnail_height    :u8
}

impl Header70 {
    fn verify(&self) -> Result<(), VTFError> {
        //Checks to see if width or hight is not a power of two
        if !(self.width.is_power_of_two() || self.height.is_power_of_two()) {
            Err(VTFError::ImageSize)
        } else {
            Ok(())
        }
    }
}

impl Data for Header70 {
    fn load<R>(source: &mut R) -> Result<Header70, VTFLoadError> where R: Read {
        use std::mem::transmute;

        let mut header70_buffer: [u8; 47] = [0; 47];
        try!(source.read(&mut header70_buffer).map_err(VTFLoadError::Io));
        let h70: Header70Raw = unsafe{ transmute(header70_buffer) };
        let header70 = unsafe{ 
            Header70 {
                width: transmute(h70.width),
                height: transmute(h70.height),
                flags: transmute(h70.flags),
                frames: transmute(h70.frames),
                start_frame: transmute(h70.start_frame),
                reflectivity: transmute(h70.reflectivity),
                bump_scale: transmute(h70.bump_scale),
                image_format: try!(ImageFormat::from_i32(transmute(h70.image_format))
                                .ok_or(VTFLoadError::VTF(VTFError::HeaderImageFormat))),
                mip_count: h70.mip_count,
                thumbnail_format: try!(ImageFormat::from_i32(transmute(h70.thumbnail_format))
                                .ok_or(VTFLoadError::VTF(VTFError::HeaderImageFormat))),
                thumbnail_width: h70.thumbnail_width,
                thumbnail_height: h70.thumbnail_height
            } 
        };
        try!(header70.verify().map_err(VTFLoadError::VTF));

        Ok(header70)
    }
}

#[derive(Debug)]
pub struct Header72 {
    pub depth           :c_ushort
}

impl Data for Header72 {
    fn load<R>(source: &mut R) -> Result<Header72, VTFLoadError> where R: Read {
        use std::mem::transmute;

        let mut header72_buffer: [u8; 2] = [0; 2];
        try!(source.read(&mut header72_buffer).map_err(VTFLoadError::Io));
        let h72: Header72Raw = unsafe{ transmute(header72_buffer) };
        let header72 = unsafe {
            Header72 {
                depth: transmute(h72.depth)
            }
        };

        Ok(header72)
    }
}

#[derive(Debug)]
pub struct Header73 {
    pub resource_count  :c_uint
}

impl Data for Header73 {
    fn load<R>(source: &mut R) -> Result<Header73, VTFLoadError> where R: Read {
        use std::mem::transmute;

        let mut header73_buffer: [u8; 7] = [0; 7];
        try!(source.read(&mut header73_buffer).map_err(VTFLoadError::Io));
        let h73: Header73Raw = unsafe{ transmute(header73_buffer) };
        let header73 = unsafe {
            Header73 {
                resource_count: transmute(h73.resource_count)
            }
        };

        Ok(header73)
    }
}


#[derive(Debug)]
pub struct Resource {
    id          :ResourceID,
    data        :u32
}

impl Data for Resource {
    fn load<R>(source: &mut R) -> Result<Self, VTFLoadError> where R: Read {
        use std::mem::transmute;

        let mut resource_buffer: [u8; 8] = [0; 8];
        try!(source.read(&mut resource_buffer).map_err(VTFLoadError::Io));
        let rsrc: ResourceRaw = unsafe{ transmute(resource_buffer) };
        let resource = unsafe {
            Resource {
                id: try!(ResourceID::from_u32(transmute(rsrc.id))
                        .ok_or(VTFLoadError::VTF(VTFError::ResourceID))),
                data: transmute(rsrc.data)
            }
        };

        Ok(resource)
    }
}

pub trait Data 
    where Self: Sized {

    fn load<R>(source: &mut R) -> Result<Self, VTFLoadError> where R: Read;
}



/// HeaderRoot as arrays of unsigned integers to assist in loading
/// Size: 16
#[derive(Default, Debug)]
#[repr(C)]
struct HeaderRootRaw {
    type_string         :[u8; 4],
    version             :[u8; 8],
    header_size         :[u8; 4]
}


/// Header70 as arrays of unsigned integers to assist in loading
/// Size: 47
#[derive(Default, Debug)]
#[repr(C)]
struct Header70Raw {
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

/// Header72 as arrays of unsigned integers to assist in loading
/// Size: 2
#[derive(Default, Debug)]
#[repr(C)]
struct Header72Raw {
    depth               :[u8; 2]
}

/// Header73 as arrays of unsigned integers to assist in loading
/// Size: 7
#[derive(Default, Debug)]
#[repr(C)]
struct Header73Raw {
    padding             :[u8; 3],
    resource_count      :[u8; 4]
}

/// Resource as arrays of unsigned integers to assist in loading
/// Size: 8
#[derive(Default, Debug)]
#[repr(C)]
struct ResourceRaw {
    id                  :[u8; 4],
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


#[derive(Debug)]
#[repr(u32)]
pub enum ResourceID {
    LegacyLowResImage       = 0x01, //make_vtf_rsrc_id(0x01, 0, 0)
    LegacyImage             = 0x30, //make_vtf_rsrc_id(0x30, 0, 0)
    Sheet                   = 0x10, //make_vtf_rsrc_id(0x10, 0, 0)
    Crc                     = 0x02435243, //make_vtf_rsrc_idf('C', 'R', 'C', RSRC_NO_DATA_CHUNK)
    TextureLODSettings      = 0x02444f4c, //make_vtf_rsrc_idf('L', 'O', 'D', RSRC_NO_DATA_CHUNK)
    TextureSettingsEx       = 0x024f5354, //make_vtf_rsrc_idf('T', 'S', 'O', RSRC_NO_DATA_CHUNK)
    KeyValueData            = 0x44564b, //make_vtf_rsrc_id('K', 'V', 'D')
    MaxDictionaryEntries    = 32, //32
}

impl FromPrimitive for ResourceID {
    fn from_i64(n: i64) -> Option<ResourceID> {
        ResourceID::from_u64(n as u64)
    }

    fn from_u64(n: u64) -> Option<ResourceID> {
        match n {
            0x01            => Some(ResourceID::LegacyLowResImage),
            0x30            => Some(ResourceID::LegacyImage),
            0x10            => Some(ResourceID::Sheet),
            0x02435243      => Some(ResourceID::Crc),
            0x02444f4c      => Some(ResourceID::TextureLODSettings),
            0x024f5354      => Some(ResourceID::TextureSettingsEx),
            0x44564b        => Some(ResourceID::KeyValueData),
            32              => Some(ResourceID::MaxDictionaryEntries),
            _               => None
        }
    }
}


/// An enum with possible image formats. Documentation taken from
/// VTFFormat.h in VTFLib by Neil Jedrzejewski & Ryan Gregg
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(i32)]
pub enum ImageFormat {
    /// Red, Green, Blue, Alpha - 32 bpp
    RGBA8888 = 0,
    /// Alpha, Blue, Green, Red - 32 bpp
    ABGR8888,
    /// Red, Green, Blue - 24 bpp
    RGB888,
    /// Blue, Green, Red - 24 bpp
    BGR888,
    /// Red, Green, Blue - 16 bpp
    RGB565,
    /// Luminance - 8 bpp
    I8,
    /// Luminance, Alpha - 16 bpp
    IA88,
    /// Paletted - 8 bpp
    P8,
    /// Alpha- 8 bpp
    A8,
    /// Red, Green, Blue, "BlueScreen" Alpha - 24 bpp
    RGB888_BLUESCREEN,
    /// Blue, Green, Red, "BlueScreen" Alpha - 24 bpp
    BGR888_BLUESCREEN,
    /// Alpha, Red, Green, Blue - 32 bpp
    ARGB8888,
    /// Blue, Green, Red, Alpha - 32 bpp
    BGRA8888,
    /// DXT1 compressed format - 4 bpp
    DXT1,
    /// DXT3 compressed format - 8 bpp
    DXT3,
    /// DXT5 compressed format - 8 bpp
    DXT5,
    /// Blue, Green, Red, Unused - 32 bpp
    BGRX8888,
    /// Blue, Green, Red - 16 bpp
    BGR565,
    /// Blue, Green, Red, Unused - 16 bpp
    BGRX5551,
    /// //!<  = Red, Green, Blue, Alpha - 16 bpp
    BGRA4444,
    /// DXT1 compressed format with 1-bit alpha - 4 bpp
    DXT1_ONEBITALPHA,
    /// Blue, Green, Red, Alpha - 16 bpp
    BGRA5551,
    /// 2 channel format for DuDv/Normal maps - 16 bpp
    UV88,
    /// 4 channel format for DuDv/Normal maps - 32 bpp
    UVWQ8888,
    /// Red, Green, Blue, Alpha - 64 bpp
    RGBA16161616F,
    /// Red, Green, Blue, Alpha signed with mantissa - 64 bpp
    RGBA16161616,
    /// 4 channel format for DuDv/Normal maps - 32 bpp
    UVLX8888,
    /// Luminance - 32 bpp
    R32F,
    /// Red, Green, Blue - 96 bpp
    RGB323232F,
    /// Red, Green, Blue, Alpha - 128 bpp
    RGBA32323232F,
    NV_DST16,
    NV_DST24,                  
    NV_INTZ,
    NV_RAWZ,
    ATI_DST16,
    ATI_DST24,
    NV_NULL,
    ATI2N,                     
    ATI1N,
    COUNT,
    NONE = -1
}

impl FromPrimitive for ImageFormat {
    fn from_i64(n: i64) -> Option<ImageFormat> {
        match n {
            0  => Some(ImageFormat::RGBA8888),
            1  => Some(ImageFormat::ABGR8888),
            2  => Some(ImageFormat::RGB888),
            3  => Some(ImageFormat::BGR888),
            4  => Some(ImageFormat::RGB565),
            5  => Some(ImageFormat::I8),
            6  => Some(ImageFormat::IA88),
            7  => Some(ImageFormat::P8),
            8  => Some(ImageFormat::A8),
            9  => Some(ImageFormat::RGB888_BLUESCREEN),
            10 => Some(ImageFormat::BGR888_BLUESCREEN),
            11 => Some(ImageFormat::ARGB8888),
            12 => Some(ImageFormat::BGRA8888),
            13 => Some(ImageFormat::DXT1),
            14 => Some(ImageFormat::DXT3),
            15 => Some(ImageFormat::DXT5),
            16 => Some(ImageFormat::BGRX8888),
            17 => Some(ImageFormat::BGR565),
            18 => Some(ImageFormat::BGRX5551),
            19 => Some(ImageFormat::BGRA4444),
            20 => Some(ImageFormat::DXT1_ONEBITALPHA),
            21 => Some(ImageFormat::BGRA5551),
            22 => Some(ImageFormat::UV88),
            23 => Some(ImageFormat::UVWQ8888),
            24 => Some(ImageFormat::RGBA16161616F),
            25 => Some(ImageFormat::RGBA16161616),
            26 => Some(ImageFormat::UVLX8888),
            27 => Some(ImageFormat::R32F),
            28 => Some(ImageFormat::RGB323232F),
            29 => Some(ImageFormat::RGBA32323232F),
            30 => Some(ImageFormat::NV_DST16),
            31 => Some(ImageFormat::NV_DST24),                  
            32 => Some(ImageFormat::NV_INTZ),
            33 => Some(ImageFormat::NV_RAWZ),
            34 => Some(ImageFormat::ATI_DST16),
            35 => Some(ImageFormat::ATI_DST24),
            36 => Some(ImageFormat::NV_NULL),
            37 => Some(ImageFormat::ATI2N),                     
            38 => Some(ImageFormat::ATI1N),
            39 => Some(ImageFormat::COUNT),
            -1 => Some(ImageFormat::NONE),
            _ => None
        }
    }

    fn from_u64(n: u64) -> Option<ImageFormat> {
        match n {
            0 ... 40 => ImageFormat::from_i64(n as i64),
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