extern crate sorcent;
extern crate image;

use std::fs::File;
use sorcent::vtf::VTFFile;
use image::png::PNGEncoder;

fn main() {
    
    let mut file = File::open("target/dirtground003.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    {
        let vtf_image = &vtf_file.image.unwrap().rgb888;
        println!("Image converted to RGB888");
        let mut png_file = File::create("target/dirtground.png").unwrap();

        let mut rgb: Vec<u8> = Vec::with_capacity(vtf_image.len() * 3);

        for c in vtf_image {
            rgb.push(c.red);
            rgb.push(c.green);
            rgb.push(c.blue);
        }

        PNGEncoder::new(&mut png_file).encode(&rgb[..], 1024, 1024, image::ColorType::RGB(8)).unwrap();
        println!("Image saved!");
    }
}