extern crate sorcent;

use std::fs::File;
use sorcent::vmt::VMTFile;

fn main() {
    let mut file = File::open("target/heavy_head_red_invun.vmt").unwrap();

    let vmt = VMTFile::open(&mut file);
    println!("{:#?}", vmt.shader);
}