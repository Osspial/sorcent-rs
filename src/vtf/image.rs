use std::io;
use std::io::Read;

use super::format::ImageFormat;

#[derive(Debug, Clone)]
pub struct Rgb565 {
    pub red: u8, // Five bits with three bits of padding
    pub green: u8, // Six bits with two bits of padding
    pub blue: u8 // Five bits with three bits of padding
}

impl Rgb565 {
    pub fn load(source: u16) -> Rgb565 {
        Rgb565 {
            red: ((source & 63488) >> 11) as u8,
            green: ((source & 2016) >> 5) as u8,
            blue: (source & 31) as u8
        }
    }
}

impl ColorType for Rgb565 {
    fn to_rgb8(&self) -> Rgb8 {
        // Conversion factor for 5-bit to 8-bit
        const CONV58: f32 = 255.0/31.0;
        // Conversion factor for 6-bit to 8-bit
        const CONV68: f32 = 255.0/63.0;

        Rgb8 {
            red: (self.red as f32 * CONV58) as u8,
            green: (self.green as f32 * CONV68) as u8,
            blue: (self.blue as f32 * CONV58) as u8,
        }
    }

    fn from_rgb888(rgb: Rgb8) -> Rgb565 {
        // Conversion factor from 8-bit to 5-bit
        const CONV85: f32 = 31.0/255.0;
        // Conversion factor from 8-bit to 6-bit
        const CONV86: f32 = 63.0/255.0;

        Rgb565 {
            red: (rgb.red as f32 * CONV85) as u8,
            green: (rgb.green as f32 * CONV86) as u8,
            blue: (rgb.blue as f32 * CONV85) as u8,
        }
    }

    fn to_rgba8(&self) -> Rgba8 {
        let rgb = self.to_rgb8();

        Rgba8 {
            red: rgb.red,
            green: rgb.green,
            blue: rgb.blue,
            alpha: 255
        }
    }

    fn from_rgba8888(rgba: Rgba8) -> Rgb565 {
        Rgb565::from_rgb888(Rgb8{red: rgba.red, green: rgba.green, blue: rgba.blue})
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Rgb8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Bgr8 {
    pub blue: u8,
    pub green: u8,
    pub red: u8
}

impl ColorType for Bgr8 {
    fn to_rgb8(&self) -> Rgb8 {
        Rgb8 {
            red: self.red,
            green: self.green,
            blue: self.blue
        }
    }

    fn from_rgb888(rgb: Rgb8) -> Bgr8 {
        Bgr8 {
            blue: rgb.blue,
            green: rgb.green,
            red: rgb.red
        }
    }

    fn to_rgba8(&self) -> Rgba8 {
        Rgba8 {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: 255
        }
    }

    fn from_rgba8888(rgba: Rgba8) -> Bgr8 {
        Bgr8 {
            blue: rgba.blue,
            green: rgba.green,
            red: rgba.red
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rgb16 {
    pub red: u16,
    pub green: u16,
    pub blue: u16
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Rgba8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Bgra8 {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8
}

impl ColorType for Bgra8 {
    fn to_rgb8(&self) -> Rgb8 {
        Rgb8 {
            red: self.red,
            green: self.green,
            blue: self.blue
        }
    }

    fn from_rgb888(rgb: Rgb8) -> Bgra8 {
        Bgra8 {
            blue: rgb.blue,
            green: rgb.green,
            red: rgb.red,
            alpha: 255
        }
    }

    fn to_rgba8(&self) -> Rgba8 {
        Rgba8 {
            red: self.red,
            green: self.green,
            blue: self.blue,
            alpha: self.alpha
        }
    }

    fn from_rgba8888(rgba: Rgba8) -> Bgra8 {
        Bgra8 {
            blue: rgba.blue,
            green: rgba.green,
            red: rgba.red,
            alpha: rgba.alpha
        }
    }
}

pub trait ColorType where Self: Sized {
    fn to_rgb8(&self) -> Rgb8;
    fn from_rgb888(rgb: Rgb8) -> Self;

    fn to_rgba8(&self) -> Rgba8;
    fn from_rgba8888(rgba: Rgba8) -> Self;
}

#[derive(Debug, Clone)]
pub enum VTFImageWrapper {
    DXT1 (Dxt1),
    DXT3 (Dxt3),
    DXT5 (Dxt5),
    BGR888 (Bgr8Image),
    BGRA8888 (Bgra8Image)
}

impl VTFImageWrapper {
    pub fn load<R>(source: &mut R, width: u16, height: u16, format: ImageFormat) -> Result<VTFImageWrapper, io::Error> where R: Read {
        match format {
            ImageFormat::DXT1 => Ok(VTFImageWrapper::DXT1(try!(Dxt1::load(&mut *source, width, height)))),
            ImageFormat::DXT3 => Ok(VTFImageWrapper::DXT3(try!(Dxt3::load(&mut *source, width, height)))),
            ImageFormat::DXT5 => Ok(VTFImageWrapper::DXT5(try!(Dxt5::load(&mut *source, width, height)))),
            ImageFormat::BGR888 => Ok(VTFImageWrapper::BGR888(try!(Bgr8Image::load(&mut *source, width, height)))),
            ImageFormat::BGRA8888 => Ok(VTFImageWrapper::BGRA8888(try!(Bgra8Image::load(&mut *source, width, height)))),
            _ => panic!("Unsupported image format given!")
        }
    }

    pub fn expose(&self) -> &VTFImage {
        match self {
            &VTFImageWrapper::DXT1(ref im) => im,
            &VTFImageWrapper::DXT3(ref im) => im,
            &VTFImageWrapper::DXT5(ref im) => im,
            &VTFImageWrapper::BGR888(ref im) => im,
            &VTFImageWrapper::BGRA8888(ref im) => im
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dxt1 {
    data: Vec<(u16, u16, [u8; 4])>,
    width: u16,
    height: u16
}

impl Dxt1 {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Dxt1, io::Error> where R: Read {
        use std::mem::transmute;

        // Internally to the VTF file format, there are no images that are
        // smaller than 4x4. This corrects for that. 
        let mut width = width;
        let mut height = height;
        if width < 4 {
            width = 4;
        }
        if height < 4 {
            height = 4;
        }
        let (width, height) = (width, height);


        let pix_count = width as usize * height as usize;
        
        let mut data: Vec<(u16, u16, [u8; 4])> = Vec::with_capacity(pix_count / 16);

        {
            let mut data_buffer: [u8; 8] = [0; 8];

            let mut index = 0;
            while index < pix_count / 2 {
                try!(source.read(&mut data_buffer));
                data.push(unsafe{ transmute(data_buffer) });
                index += 8;
            }
        }

        Ok(Dxt1 {data: data, width: width, height: height})
    }
}

impl VTFImage for Dxt1 {
    fn to_rgb8(&self) -> Vec<Rgb8> {

        let pix_count = self.width as usize * self.height as usize;
        let mut rgb: Vec<Rgb8> = Vec::with_capacity(pix_count);
        unsafe{ rgb.set_len(pix_count) };

        let mut chunk_offset = 0;
        for c in &self.data {
            let c0 = Rgb565::load(c.0).to_rgb8();
            let c1 = Rgb565::load(c.1).to_rgb8();
            let c2 = interp_color(&c0, &c1, true);
            let c3 = interp_color(&c0, &c1, false); 


            let color_data: [u8; 16] = [c.2[0] & 3, c.2[0] >> 2 & 3, c.2[0] >> 4 & 3, c.2[0] >> 6 & 3,
                                        c.2[1] & 3, c.2[1] >> 2 & 3, c.2[1] >> 4 & 3, c.2[1] >> 6 & 3,
                                        c.2[2] & 3, c.2[2] >> 2 & 3, c.2[2] >> 4 & 3, c.2[2] >> 6 & 3,
                                        c.2[3] & 3, c.2[3] >> 2 & 3, c.2[3] >> 4 & 3, c.2[3] >> 6 & 3];

            // The index for adding data to the chunk array
            let mut index: usize = 0;
            // The line of the chunk to write data to
            let mut cline = 0;
            for co in &color_data {
                match *co {
                    0 => rgb[chunk_offset + index + (self.width*cline) as usize] = c0.clone(),
                    1 => rgb[chunk_offset + index + (self.width*cline) as usize] = c1.clone(),
                    2 => rgb[chunk_offset + index + (self.width*cline) as usize] = c2.clone(),
                    3 => rgb[chunk_offset + index + (self.width*cline) as usize] = c3.clone(),
                    _ => unreachable!()
                }

                index += 1;
                if index > 3 {
                    index = 0;
                    cline += 1;
                }
            }
            
            chunk_offset += 4;
            let wusize = self.width as usize;
            if chunk_offset % wusize == 0 && chunk_offset >= wusize {
                chunk_offset += wusize * 3;
            }
        }

        rgb
    }

    fn to_rgba8(&self) -> Vec<Rgba8> {
        let rgb = self.to_rgb8();

        let mut rgba: Vec<Rgba8> = Vec::with_capacity(rgb.len());
        for c in &rgb {
            rgba.push(Rgba8{red: c.red, green: c.green, blue: c.blue, alpha: 255});
        }

        rgba
    }

    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone)]
pub struct Dxt3 {
    data: Vec<([u8; 8], u16, u16, [u8; 4])>,
    width: u16,
    height: u16
}

impl Dxt3 {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Dxt3, io::Error> where R: Read {
        use std::mem::transmute;

        // Internally to the VTF file format, there are no images that are
        // smaller than 4x4. This corrects for that. 
        let mut width = width;
        let mut height = height;
        if width < 4 {
            width = 4;
        }
        if height < 4 {
            height = 4;
        }
        let (width, height) = (width, height);


        let pix_count = width as usize * height as usize;
        
        // Capacity is pix_count / 16 as that is the number of pixel chunks that the image gets
        // compressed into. The reason that is the number is that each pixel chunk contains
        // 16 pixels.
        let mut data: Vec<([u8; 8], u16, u16, [u8; 4])> = Vec::with_capacity(pix_count / 16);

        {
            let mut data_buffer: [u8; 16] = [0; 16];

            let mut index = 0;
            while index < pix_count {
                try!(source.read(&mut data_buffer));
                data.push(unsafe{ transmute(data_buffer) });
                index += 16;
            }
        }

        Ok(Dxt3 {data: data, width: width, height: height})
    }
}

impl VTFImage for Dxt3 {
    fn to_rgb8(&self) -> Vec<Rgb8> {
        let rgba = self.to_rgba8();

        let mut rgb: Vec<Rgb8> = Vec::with_capacity(rgba.len());
        for c in &rgba {
            rgb.push(Rgb8{red: c.red, green: c.green, blue: c.blue});
        }

        rgb
    }

    fn to_rgba8(&self) -> Vec<Rgba8> {

        let pix_count = self.width as usize * self.height as usize;
        let mut rgba: Vec<Rgba8> = Vec::with_capacity(pix_count);
        unsafe{ rgba.set_len(pix_count) };

        let mut chunk_offset = 0;
        for c in &self.data {
            // Compute color data
            let c0 = Rgb565::load(c.1).to_rgb8();
            let c1 = Rgb565::load(c.2).to_rgb8();
            let c2 = interp_color(&c0, &c1, true);
            let c3 = interp_color(&c0, &c1, false); 

            let color_data: [u8; 16] = [c.3[0] & 3, c.3[0] >> 2 & 3, c.3[0] >> 4 & 3, c.3[0] >> 6 & 3,
                                        c.3[1] & 3, c.3[1] >> 2 & 3, c.3[1] >> 4 & 3, c.3[1] >> 6 & 3,
                                        c.3[2] & 3, c.3[2] >> 2 & 3, c.3[2] >> 4 & 3, c.3[2] >> 6 & 3,
                                        c.3[3] & 3, c.3[3] >> 2 & 3, c.3[3] >> 4 & 3, c.3[3] >> 6 & 3];


            let alpha_data: [u8; 16] = [c.0[0] & 15, c.0[0] >> 4 & 15, c.0[1] & 15, c.0[1] >> 4 & 15,
                                        c.0[2] & 15, c.0[2] >> 4 & 15, c.0[3] & 15, c.0[3] >> 4 & 15,
                                        c.0[4] & 15, c.0[4] >> 4 & 15, c.0[5] & 15, c.0[5] >> 4 & 15,
                                        c.0[6] & 15, c.0[6] >> 4 & 15, c.0[7] & 15, c.0[7] >> 4 & 15];

            // The index for adding data to the chunk array
            let mut index: usize = 0;
            // The line of the chunk to write data to
            let mut cline = 0;
            let mut i = 0;
            while i < 16 {
                let rgba_offset = chunk_offset + index + (self.width*cline) as usize;
                match color_data[i] {
                    0 => rgba[rgba_offset] = Rgba8{red: c0.red, green: c0.green, blue: c0.blue, alpha: alpha_data[i] * 17},
                    1 => rgba[rgba_offset] = Rgba8{red: c1.red, green: c1.green, blue: c1.blue, alpha: alpha_data[i] * 17},
                    2 => rgba[rgba_offset] = Rgba8{red: c2.red, green: c2.green, blue: c2.blue, alpha: alpha_data[i] * 17},
                    3 => rgba[rgba_offset] = Rgba8{red: c3.red, green: c3.green, blue: c3.blue, alpha: alpha_data[i] * 17},
                    _ => unreachable!()
                }
                

                index += 1;
                if index > 3 {
                    index = 0;
                    cline += 1;
                }
                i += 1;
            }
            
            chunk_offset += 4;
            let wusize = self.width as usize;
            if chunk_offset % wusize == 0 && chunk_offset >= wusize {
                chunk_offset += wusize * 3;
            }
        }

        rgba
    }

    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone)]
pub struct Dxt5 {
    data: Vec<(u8, u8, [u8; 6], u16, u16, [u8; 4])>,
    width: u16,
    height: u16
}

impl Dxt5 {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Dxt5, io::Error> where R: Read {
        use std::mem::transmute;

        // Internally to the VTF file format, there are no images that are
        // smaller than 4x4. This corrects for that. 
        let mut width = width;
        let mut height = height;
        if width < 4 {
            width = 4;
        }
        if height < 4 {
            height = 4;
        }
        let (width, height) = (width, height);


        let pix_count = width as usize * height as usize;
        
        // Capacity is pix_count / 16 as that is the number of pixel chunks that the image gets
        // compressed into. The reason that is the number is that each pixel chunk contains
        // 16 pixels.
        let mut data: Vec<(u8, u8, [u8; 6], u16, u16, [u8; 4])> = Vec::with_capacity(pix_count / 16);

        {
            let mut data_buffer: [u8; 16] = [0; 16];

            let mut index = 0;
            while index < pix_count {
                try!(source.read(&mut data_buffer));
                data.push(unsafe{ transmute(data_buffer) });
                index += 16;
            }
        }

        Ok(Dxt5 {data: data, width: width, height: height})
    }
}

impl VTFImage for Dxt5 {
    fn to_rgb8(&self) -> Vec<Rgb8> {
        let rgba = self.to_rgba8();

        let mut rgb: Vec<Rgb8> = Vec::with_capacity(rgba.len());
        for c in &rgba {
            rgb.push(Rgb8{red: c.red, green: c.green, blue: c.blue});
        }

        rgb
    }

    fn to_rgba8(&self) -> Vec<Rgba8> {

        let pix_count = self.width as usize * self.height as usize;
        let mut rgba: Vec<Rgba8> = Vec::with_capacity(pix_count);
        unsafe{ rgba.set_len(pix_count) };

        let mut chunk_offset = 0;
        for c in &self.data {
            // Compute color data
            let c0 = Rgb565::load(c.3).to_rgb8();
            let c1 = Rgb565::load(c.4).to_rgb8();
            let c2 = interp_color(&c0, &c1, true);
            let c3 = interp_color(&c0, &c1, false); 

            let color_data: [u8; 16] = [c.5[0] & 3, c.5[0] >> 2 & 3, c.5[0] >> 4 & 3, c.5[0] >> 6 & 3,
                                        c.5[1] & 3, c.5[1] >> 2 & 3, c.5[1] >> 4 & 3, c.5[1] >> 6 & 3,
                                        c.5[2] & 3, c.5[2] >> 2 & 3, c.5[2] >> 4 & 3, c.5[2] >> 6 & 3,
                                        c.5[3] & 3, c.5[3] >> 2 & 3, c.5[3] >> 4 & 3, c.5[3] >> 6 & 3];

            // Compute alpha data
            let a0 = c.0;
            let a1 = c.1;
            // Array of the raw, interpolated alpha values
            let alookup: [u8; 8];

            // Note: the following if/else statement is adapted from a
            // similar alpha computation statement in VTFLib's VTFFile.cpp

            // 8-alpha or 6-alpha block?    
            if a0 > a1 {

                // 8-bit alpha block.
                // Bit code 000 = a0, 001 = a1, others are interpolated.
                alookup = [
                    a0,
                    a1,
                    interp_alpha_8bit(a0, a1, 0),
                    interp_alpha_8bit(a0, a1, 1),
                    interp_alpha_8bit(a0, a1, 2),
                    interp_alpha_8bit(a0, a1, 3),
                    interp_alpha_8bit(a0, a1, 4),
                    interp_alpha_8bit(a0, a1, 5)
                ];

            } else {  

                // 6-alpha block.    
                // Bit code 000 = alpha_0, 001 = alpha_1, others are interpolated.
                alookup = [
                    a0,
                    a1,
                    interp_alpha_6bit(a0, a1, 0),
                    interp_alpha_6bit(a0, a1, 1),
                    interp_alpha_6bit(a0, a1, 2),
                    interp_alpha_6bit(a0, a1, 3),
                    0x00,
                    0xFF
                ];
            }

            let alpha_data: [u8; 16] = {
                use std::mem::transmute;

                let alpha: u64 = unsafe{ transmute([c.2[0], c.2[1], c.2[2], c.2[3], c.2[4], c.2[5], 00, 00]) };
                [(alpha & 7)       as u8, (alpha >> 3 & 7)  as u8, (alpha >> 6 & 7)  as u8, (alpha >> 9 & 7)  as u8,
                 (alpha >> 11 & 7) as u8, (alpha >> 15 & 7) as u8, (alpha >> 18 & 7) as u8, (alpha >> 21 & 7) as u8,
                 (alpha >> 24 & 7) as u8, (alpha >> 27 & 7) as u8, (alpha >> 30 & 7) as u8, (alpha >> 33 & 7) as u8,
                 (alpha >> 36 & 7) as u8, (alpha >> 39 & 7) as u8, (alpha >> 42 & 7) as u8, (alpha >> 45 & 7) as u8]
                
            };

            // The index for adding data to the chunk array
            let mut index: usize = 0;
            // The line of the chunk to write data to
            let mut cline = 0;
            let mut i = 0;
            while i < 16 {
                let rgba_offset = chunk_offset + index + (self.width*cline) as usize;
                match color_data[i] {
                    0 => rgba[rgba_offset] = Rgba8{red: c0.red, green: c0.green, blue: c0.blue, alpha: alookup[alpha_data[i] as usize]},
                    1 => rgba[rgba_offset] = Rgba8{red: c1.red, green: c1.green, blue: c1.blue, alpha: alookup[alpha_data[i] as usize]},
                    2 => rgba[rgba_offset] = Rgba8{red: c2.red, green: c2.green, blue: c2.blue, alpha: alookup[alpha_data[i] as usize]},
                    3 => rgba[rgba_offset] = Rgba8{red: c3.red, green: c3.green, blue: c3.blue, alpha: alookup[alpha_data[i] as usize]},
                    _ => unreachable!()
                }
                

                index += 1;
                if index > 3 {
                    index = 0;
                    cline += 1;
                }
                i += 1;
            }
            
            chunk_offset += 4;
            let wusize = self.width as usize;
            if chunk_offset % wusize == 0 && chunk_offset >= wusize {
                chunk_offset += wusize * 3;
            }
        }

        rgba
    }

    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone)]
pub struct Bgr8Image {
    data: Vec<Bgr8>,
    width: u16,
    height: u16
}

impl Bgr8Image {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Bgr8Image, io::Error> where R: Read {
        use std::mem::transmute;

        let pix_count = width as usize * height as usize;

        let mut data: Vec<Bgr8> = Vec::with_capacity(pix_count);


        let mut data_buffer: [u8; 3] = [0; 3];
        let mut index = 0;
        while index < pix_count {
            try!(source.read(&mut data_buffer));
            data.push(unsafe{ transmute(data_buffer) });
            index += 1;
        }

        Ok(Bgr8Image{data: data, width: width, height: height})
    }
}

impl VTFImage for Bgr8Image {
    fn to_rgb8(&self) -> Vec<Rgb8> {
        let pix_count = self.width as usize * self.height as usize;

        let mut rgb: Vec<Rgb8> = Vec::with_capacity(pix_count);

        for p in &self.data {
            rgb.push(p.to_rgb8());
        }

        rgb
    }

    fn to_rgba8(&self) -> Vec<Rgba8> {
        let pix_count = self.width as usize * self.height as usize;

        let mut rgba: Vec<Rgba8> = Vec::with_capacity(pix_count);

        for p in &self.data {
            rgba.push(p.to_rgba8());
        }

        rgba
    }

    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }
}

#[derive(Debug, Clone)]
pub struct Bgra8Image {
    data: Vec<Bgra8>,
    width: u16,
    height: u16
}

impl Bgra8Image {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Bgra8Image, io::Error> where R: Read {
        use std::mem::transmute;

        let pix_count = width as usize * height as usize;

        let mut data: Vec<Bgra8> = Vec::with_capacity(pix_count);


        let mut data_buffer: [u8; 4] = [0; 4];
        let mut index = 0;
        while index < pix_count {
            try!(source.read(&mut data_buffer));
            data.push(unsafe{ transmute(data_buffer) });
            index += 1;
        }

        Ok(Bgra8Image{data: data, width: width, height: height})
    }
}

impl VTFImage for Bgra8Image {
    fn to_rgb8(&self) -> Vec<Rgb8> {
        let pix_count = self.width as usize * self.height as usize;

        let mut rgb: Vec<Rgb8> = Vec::with_capacity(pix_count);

        for p in &self.data {
            rgb.push(p.to_rgb8());
        }

        rgb
    }

    fn to_rgba8(&self) -> Vec<Rgba8> {
        let pix_count = self.width as usize * self.height as usize;

        let mut rgba: Vec<Rgba8> = Vec::with_capacity(pix_count);

        for p in &self.data {
            rgba.push(p.to_rgba8());
        }

        rgba
    }

    fn get_width(&self) -> u16 {
        self.width
    }

    fn get_height(&self) -> u16 {
        self.height
    }
}



pub trait VTFImage {
    fn to_rgb8(&self) -> Vec<Rgb8>;
    fn to_rgb8_raw(&self) -> Vec<u8> {
        let rgb = self.to_rgb8();

        let mut rgb_raw = Vec::with_capacity(rgb.len() * 3);
        for p in &rgb {
            rgb_raw.push(p.red);
            rgb_raw.push(p.green);
            rgb_raw.push(p.blue);
        }
        rgb_raw

        /*
        use std::mem;

        let mut rgb = self.to_rgb8();
        unsafe{
            let ptr = rgb.as_mut_ptr();
            let len = rgb.len() * 3;
            let cap = rgb.capacity() * 3;

            Vec::from_raw_parts(mem::transmute(ptr), len, cap)
        }
        */
    }

    fn to_rgba8(&self) -> Vec<Rgba8>;
    fn to_rgba8_raw(&self) -> Vec<u8> {
        let rgba = self.to_rgba8();

        let mut rgba_raw = Vec::with_capacity(rgba.len() * 4);
        for p in &rgba {
            rgba_raw.push(p.red);
            rgba_raw.push(p.green);
            rgba_raw.push(p.blue);
            rgba_raw.push(p.alpha);
        }
        rgba_raw

        /*
        use std::mem;

        let mut rgba = self.to_rgba8();
        unsafe{
            let ptr = rgba.as_mut_ptr();
            let len = rgba.len() * 4;
            let cap = rgba.capacity() * 4;

            Vec::from_raw_parts(mem::transmute(ptr), len, cap)
        }
        */
    }

    fn get_width(&self) -> u16;
    fn get_height(&self) -> u16;
}

/// Interpolates between colors c0 and c1. When factor is false,
/// the output color is set to 2/3 c0 + 1/3 c1. When factor is
/// true, the output color is set to 1/3 c0 + 2/3 c1
fn interp_color(c0: &Rgb8, c1: &Rgb8, factor: bool) -> Rgb8 {
    let c0 = Rgb16 {red: c0.red as u16, green: c0.green as u16, blue: c0.blue as u16};
    let c1 = Rgb16 {red: c1.red as u16, green: c1.green as u16, blue: c1.blue as u16};

    match factor {
        true =>     Rgb8 {
                        red: ((2 * c0.red + c1.red) / 3) as u8,
                        green: ((2 * c0.green + c1.green) / 3) as u8,
                        blue: ((2 * c0.blue + c1.blue) / 3) as u8
                    },
        false =>    Rgb8 {
                        red: ((c0.red + 2 * c1.red) / 3) as u8,
                        green: ((c0.green + 2 * c1.green) / 3) as u8,
                        blue: ((c0.blue + 2 * c1.blue) / 3) as u8,
                    }
    }
}

/// Interpolates between alphas a0 and a1. Factor is on a scale of
/// 0-3, with 0 being mostly a0 and 3 being mostly a1. Anything
/// outside of that range has undefined behavior.
#[inline]
fn interp_alpha_8bit(a0: u8, a1: u8, factor: u8) -> u8 {
    let a0 = a0 as u32;
    let a1 = a1 as u32;
    let factor = factor as u32;
    (((6-factor) * a0 + (1+factor) * a1 + 3) / 7) as u8
}

/// Interpolates between alphas a0 and a1. Factor is on a scale of
/// 0-5, with 0 being mostly a0 and 5 being mostly a1. Anything
/// outside of that range has undefined behavior.
#[inline]
fn interp_alpha_6bit(a0: u8, a1: u8, factor: u8) -> u8 {
    let a0 = a0 as u16;
    let a1 = a1 as u16;
    let factor = factor as u16;

    (((4-factor) * a0 + (1+factor) * a1 + 2) / 5) as u8
}