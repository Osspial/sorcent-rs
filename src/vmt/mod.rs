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
use self::format::Shader;

pub struct VMTFile<'s> {
    // A paddingless string containing all of the strings in the vmt.
    // Used to create slices for the various shader elements.
    vmt_els: String,
    shader: Shader<'s>
}


impl<'s> VMTFile<'s> {
    pub fn open(file: &mut File) {
        let mut buf_read = BufReader::new(file);
        let mut vmt_string = String::new();

        buf_read.read_to_string(&mut vmt_string).unwrap();
        let lexer = Lexer::new(&vmt_string[..]).unwrap();

        let mut element_len = 0;
        let mut element_num = 0;
        for t in &lexer.tokens {
            match t.get_inner_str() {
                Some(s) => {element_len += s.len();
                            element_num += 1},
                None    => ()
            }
        }

        let mut vmt_elements = String::with_capacity(element_len);
        // A vector of the lengths of each string slice in the vmt_elements string
        let mut element_lens = Vec::with_capacity(element_num);
        for t in &lexer.tokens {
            match t.get_inner_str() {
                Some(s) => {vmt_elements.push_str(s);
                            element_lens.push(s.len())},
                None    => ()
            }
        }

        let shader = Shader::from_raw_parts(&lexer.tokens, &vmt_elements[..], &element_lens).unwrap();

        println!("{:#?}", shader);
    }
}
