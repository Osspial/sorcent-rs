extern crate sorcent;
extern crate image;

use std::fs::File;
use sorcent::vtf::VTFFile;
use image::png::PNGEncoder;

fn main() {
    
    let mut file = File::open("target/officedoor_04.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    {
        let vtf_image = &vtf_file.image.to_rgb888();
        println!("Image converted to RGB888");
        let mut png_file = File::create("target/officedoor_04.png").unwrap();

        let mut rgb: Vec<u8> = Vec::with_capacity(vtf_image.len() * 3);

        for c in vtf_image {
            rgb.push(c.red);
            rgb.push(c.green);
            rgb.push(c.blue);
        }

        PNGEncoder::new(&mut png_file).encode(&rgb[..], 256, 512, image::ColorType::RGB(8)).unwrap();
        println!("Image saved!");
    }
}