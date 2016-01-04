utse std::fs::File;

#[allow(dead_code)]
mod format;


pub fn open(path: &str) {
    let mut file = File::open(path).unwrap();
    let header70 = format::Header70::open(&mut file).unwrap();

    println!("{:#?}", header70);
}
