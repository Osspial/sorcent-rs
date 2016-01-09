use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct Rgb565 {
    pub red: u8, //Five bits with three bits of padding
    pub green: u8, //Six bits with two bits of padding
    pub blue: u8 //Five bits with three bits of padding
}

impl Rgb565 {
    pub fn load(source: u16) -> Rgb565{
        Rgb565 {
            red: ((source & 63488) >> 11) as u8,
            green: ((source & 2016) >> 5) as u8,
            blue: (source & 31) as u8
        }
    }

    pub fn to_rgb888(&self) -> Rgb888 {

        //Conversion factor for 5-bit to 8-bit
        const CONV58: f32 = 255.0/31.0;
        //Conversion factor for 6-bit to 8-bit
        const CONV68: f32 = 255.0/63.0;

        Rgb888 {
            red: (self.red as f32 * CONV58) as u8,
            green: (self.green as f32 * CONV68) as u8,
            blue: (self.blue as f32 * CONV58) as u8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Rgb888 {
    pub red: u8,
    pub green: u8,
    pub blue: u8
}

#[derive(Debug, Clone)]
pub struct Rgb161616 {
    pub red: u16,
    pub green: u16,
    pub blue: u16
}

pub struct Rgba8888 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8
}

#[derive(Debug)]
pub struct Dxt1 {
    pub data: Vec<(u16, u16, [u8; 4])>,
    width: u16,
    height: u16
}

impl Dxt1 {
    pub fn load<R>(source: &mut R, width: u16, height: u16) -> Result<Dxt1, io::Error> where R: Read {
        use std::mem::transmute;

        // Internally to the VTF file format, there are no images that are
        // smaller than 4. This corrects for that. 
        let mut width = width;
        let mut height = height;
        if width < 4 {
            width = 4;
        }
        if height < 4 {
            height = 4;
        }
        let width = width;
        let height = height;

        let pix_count = width as usize * height as usize;
        
        let mut data: Vec<(u16, u16, [u8; 4])> = Vec::with_capacity(pix_count / 16);

        {
            let mut data_buffer: [u8; 8] = [0; 8];

            let mut index = 0;
            while index < pix_count / 2{
                try!(source.read(&mut data_buffer));
                data.push(unsafe{ transmute(data_buffer) });
                index += 8;
            }
        }

        Ok(Dxt1 {data: data, width: width, height: height})
    }

    pub fn to_rgb888(&self) -> Vec<Rgb888> {

        let pix_count = self.width as usize * self.height as usize;
        let mut rgb: Vec<Rgb888> = Vec::with_capacity(pix_count);
        unsafe{ rgb.set_len(pix_count) };

        let mut chunk_offset = 0;
        for c in &self.data {
            let c0 = Rgb565::load(c.0).to_rgb888();
            let c1 = Rgb565::load(c.1).to_rgb888();
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
                match *co & 3 {
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
}

/// Interpolates between colors c0 and c1. When factor is false,
/// the output color is set to 2/3 c0 + 1/3 c1. When factor is
/// true, the output color is set to 1/3 c0 + 2/3 c1
fn interp_color(c0: &Rgb888, c1: &Rgb888, factor: bool) -> Rgb888{
    let c0 = Rgb161616 {red: c0.red as u16, green: c0.green as u16, blue: c0.blue as u16};
    let c1 = Rgb161616 {red: c1.red as u16, green: c1.green as u16, blue: c1.blue as u16};

    match factor {
        true =>     Rgb888 {
                        red: ((2 * c0.red + c1.red) / 3) as u8,
                        green: ((2 * c0.green + c1.green) / 3) as u8,
                        blue: ((2 * c0.blue + c1.blue) / 3) as u8
                    },
        false =>    Rgb888 {
                        red: ((c0.red + 2 * c1.red) / 3) as u8,
                        green: ((c0.green + 2 * c1.green) / 3) as u8,
                        blue: ((c0.blue + 2 * c1.blue) / 3) as u8,
                    }
    }
}