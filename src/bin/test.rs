extern crate srcengt;

use std::fs::File;

fn main() {
    let mut file = File::open("wood_wall001.vtf").unwrap();
    let vtf_file = srcengt::vtf::VTFFile::open(&mut file).unwrap();

    println!("{:#?}", vtf_file);
}