extern crate sorcent;
extern crate image;

use std::fs::File;
use std::io::Read;
use sorcent::vtf::VTFFile;
use image::png::PNGEncoder;

fn main() {
    
    let mut file = File::open("target/blood1.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    {
        let vtf_image = &vtf_file.image.to_rgba8888().unwrap();
        println!("Image converted to RGBA8888");
        let mut png_file = File::create("target/blood.png").unwrap();

        let mut rgb: Vec<u8> = Vec::with_capacity(vtf_image.len() * 3);

        for c in vtf_image {
            rgb.push(c.red);
            rgb.push(c.green);
            rgb.push(c.blue);
            rgb.push(c.alpha);
        }

        PNGEncoder::new(&mut png_file).encode(&rgb[..], 128, 128, image::ColorType::RGBA(8)).unwrap();
        println!("Image saved!");
    }

    let mut end = Vec::with_capacity(512);
    file.read_to_end(&mut end).unwrap();

    println!("{}", end.len());
}