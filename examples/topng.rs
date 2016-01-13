extern crate sorcent;
extern crate image;

use std::fs::File;
use std::io::Read;
use sorcent::vtf::VTFFile;
use sorcent::vtf::image::VTFImage;
use image::png::PNGEncoder;

fn main() {
    let mut file = File::open("target/bgrtest.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    
    let vtf_image = vtf_file.image.expose();
    let rgb = vtf_image.to_rgba8888_raw();
    println!("Image converted to RGB888");
    let mut png_file = File::create("target/bgrtest.png").unwrap();

    PNGEncoder::new(&mut png_file).encode(&rgb[..], vtf_image.get_width() as u32, vtf_image.get_height() as u32, image::ColorType::RGBA(8)).unwrap();
    println!("Image saved!");

    let mut end: Vec<u8> = Vec::new();
    file.read_to_end(&mut end).unwrap();

    println!("{}", end.len());
}