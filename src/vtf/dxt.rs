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
pub struct Dxt1Raw {
    pub data: Vec<(u16, u16, [u8; 4])>,
    pub rgb888: Vec<Rgb888>
}

impl Dxt1Raw {
    pub fn load<R>(source: &mut R, length: usize) -> Result<Dxt1Raw, io::Error> where R: Read {
        use std::mem;
        use std::mem::transmute;

        let mut data_raw: Vec<u8> = Vec::with_capacity(length);
        unsafe{ data_raw.set_len(length) };
        try!(source.read(unsafe{ transmute(&mut data_raw[..]) }));
        
        // Reinterprets the bytes read into data_raw as a vector of pixel chunks
        let data: Vec<(u16, u16, [u8; 4])> = unsafe{
                                                    let p = data_raw.as_mut_ptr();
                                                    let len = data_raw.len();
                                                    let cap = data_raw.capacity();

                                                    mem::forget(data_raw);
                                                    Vec::from_raw_parts(transmute(p), len / 16, cap / 16)
                                                };

        let mut rgb: Vec<Rgb888> = Vec::with_capacity(length * 2);
        unsafe{ rgb.set_len(length * 2) };
        let line_len = ((length * 2) as f32).sqrt() as usize;

        for c in &data {
            let c0 = Rgb565::load(c.0).to_rgb888();
            let c3 = Rgb565::load(c.1).to_rgb888();
            let c1 = interp_color(&c0, &c3, true);
            let c2 = interp_color(&c0, &c3, false); 

            let color_data: [u8; 16] = [c.2[0] >> 6, c.2[0] >> 4 & 3, c.2[0] >> 2 & 3, c.2[0] & 3,
                                        c.2[1] >> 6, c.2[1] >> 4 & 3, c.2[1] >> 2 & 3, c.2[1] & 3,
                                        c.2[2] >> 6, c.2[2] >> 4 & 3, c.2[2] >> 2 & 3, c.2[2] & 3,
                                        c.2[3] >> 6, c.2[3] >> 4 & 3, c.2[3] >> 2 & 3, c.2[3] & 3];

            // The index for adding data to the chunk array
            let mut index = 0;
            // The line of the chunk to write data to
            let mut cline = 0;
            for co in color_data.into_iter() {
                print!("{:b} ", *co);
                match *co & 3 {
                    0 => rgb[index + line_len*cline] = c0.clone(),
                    1 => rgb[index + line_len*cline] = c1.clone(),
                    2 => rgb[index + line_len*cline] = c2.clone(),
                    3 => rgb[index + line_len*cline] = c3.clone(),
                    _ => unreachable!()
                }

                index += 1;
                if index > 3 {
                    index = 0;
                    cline += 1;
                }
            }
            println!("");
        }

        Ok(Dxt1Raw {data: data, rgb888: rgb})
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