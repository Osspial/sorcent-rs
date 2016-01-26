extern crate sorcent;

use std::fs::File;
use sorcent::vmt::VMTFile;

fn main() {
    let mut file = File::open("files/test.vmf").unwrap();

    let vmt = VMTFile::open(&mut file);
    println!("{:#?}", vmt.shader);
}