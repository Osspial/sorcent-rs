use std::fs::File;

#[allow(dead_code)]
mod format;


pub fn open(path: &str) {
    let mut file = File::open(path).unwrap();
    let header73 = format::Header73::open(&mut file).unwrap();

    println!("{:#?}", header73);
}
