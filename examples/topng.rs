extern crate sorcent;
extern crate image;

use std::fs::File;
use sorcent::vtf::VTFFile;
use image::png::PNGEncoder;

fn main() {
    let mut file = File::open("target/blood1.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    
    //let vtf_image = vtf_file.image.to_rgba8888().unwrap();
    let rgb = vtf_file.image.to_rgba8888_raw().unwrap();
    println!("Image converted to RGBA8888");
    let mut png_file = File::create("target/blood.png").unwrap();

    PNGEncoder::new(&mut png_file).encode(&rgb[..], 128, 128, image::ColorType::RGBA(8)).unwrap();
    println!("Image saved!");
}