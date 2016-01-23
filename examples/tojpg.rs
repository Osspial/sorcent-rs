extern crate sorcent;
extern crate image;

use std::fs::File;
use std::io::BufWriter;
use sorcent::vtf::VTFFile;
use sorcent::vtf::image::VTFImage;
use image::jpeg::JPEGEncoder;

fn main() {
    let mut file = File::open("target/concretefloor003.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();
    
    let vtf_image = vtf_file.image.expose();
    let rgb = vtf_image.to_rgb8_raw();
    println!("Image converted to RGB888");
    let jpg_file = File::create("target/concretefloor.jpg").unwrap();

    JPEGEncoder::new(&mut jpg_file).encode(&rgb[..], vtf_image.get_width() as u32, vtf_image.get_height() as u32, image::ColorType::RGB(8)).unwrap();
    println!("Image saved!");
}