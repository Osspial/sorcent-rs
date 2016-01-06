use std::fs::File;
use std::mem;

#[allow(dead_code)]
mod format;

use self::format::Header;


pub fn open(path: &str) {
    let mut file = File::open(path).unwrap();
    let metadata = file.metadata().unwrap();

    if metadata.is_dir() {
        panic!("Found directory, expected file");
    }

    //Corruption checks
    if metadata.len() < mem::size_of::<format::Header70>() as u64 {
        panic!("File too small for it's header!");
    }

    let header73 = format::Header73::open(&mut file).unwrap();

    println!("{:#?}", header73);
}
