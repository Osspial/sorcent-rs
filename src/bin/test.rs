extern crate sorcent;
extern crate image;

use std::fs::File;
use std::io::Read;
use sorcent::vtf::VTFFile;
use image::png::PNGEncoder;

fn main() {
    
    let mut file = File::open("target/wood_wall001.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    //println!("{:#?}", vtf_file);
    {
        let vtf_image = &vtf_file.image.unwrap().rgb888;
        let mut png_file = File::create("target/wood_wall.png").unwrap();
        println!("almost");

        println!("{}", vtf_image.len());
        let mut rgb: Vec<u8> = Vec::with_capacity(vtf_image.len() * 3);

        for c in vtf_image {
            rgb.push(c.red);
            rgb.push(c.green);
            rgb.push(c.blue);
        }

        PNGEncoder::new(&mut png_file).encode(&rgb[..], 1024, 1024, image::ColorType::RGB(8)).unwrap();
        println!("finally");
    }

    let mut end_buffer: Vec<u8> = Vec::with_capacity(512);

    file.read_to_end(&mut end_buffer).unwrap();
    println!("{}", end_buffer.len());
}