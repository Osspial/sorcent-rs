extern crate sorcent;
extern crate image;

use std::fs::File;
use sorcent::vtf::VTFFile;

fn main() {
    
    let mut file = File::open("wood_wall001.vtf").unwrap();
    let vtf_file = VTFFile::open(&mut file).unwrap();

    println!("{:#?}", vtf_file);
    println!("{}", vtf_file.image.unwrap().rgb888.len());	
}