#[allow(dead_code)]
pub mod format;
#[allow(dead_code)]
mod lexer;
#[allow(dead_code)]
pub mod types;
pub mod error;

use std::fs::File;
use std::io::{Read, BufReader};

use self::lexer::Lexer;

pub struct VMTFile<'s> {
    vmt_str: String,
    lexer: Lexer<'s>
}


impl<'s> VMTFile<'s> {
    pub fn open(file: &mut File) {
        let mut buf_read = BufReader::new(file);
        let mut vmt_string = String::new();

        buf_read.read_to_string(&mut vmt_string).unwrap();
        let lexer = Lexer::new(&vmt_string[..]).unwrap();

        println!("{:#?}", lexer.tokens);
    }
}
