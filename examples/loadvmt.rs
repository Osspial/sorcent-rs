extern crate sorcent;

use std::fs::File;
use sorcent::vmt::VMTFile;

fn main() {
    let mut file = File::open("target/moon_wallpanels02.vmt").unwrap();

    let vmt = VMTFile::open(&mut file);
    println!("{:#?}", vmt.shader);
    /* //This stuff will crash the program
    let vmt_slice = {
        let vmt = VMTFile::open(&mut file).unwrap();
        let parameter = &vmt.get_shader().get_parameters()[0];
        parameter.clone().get_type()
    };

    let mut vec: Vec<u8> = Vec::with_capacity(2048);
    unsafe {vec.set_len(2048)};
    for c in vec {
        print!("{}", c as char);
    }
    println!("");

    println!("{:#?}", vmt_slice);
    */
}