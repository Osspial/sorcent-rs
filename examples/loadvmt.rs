extern crate sorcent;

use std::fs::File;
use sorcent::vmt::VMTFile;

fn main() {
    let mut file = File::open("target/moon_wallpanels02.vmt").unwrap();

    let vmt = VMTFile::open(&mut file);
    println!("{:#?}", vmt.shader);
}