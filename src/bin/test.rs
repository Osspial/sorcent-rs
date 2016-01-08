extern crate srcengt;
extern crate image;

use std::fs::File;
use srcengt::vtf::VTFFile;
use srcengt::vtf::dxt::{Rgb565, Rgb888};

fn main() {
    
    let mut file = File::open("wood_wall001.vtf").unwrap();
    let vtf_file = srcengt::vtf::VTFFile::open(&mut file).unwrap();

    //println!("{:#?}", vtf_file);
    
}