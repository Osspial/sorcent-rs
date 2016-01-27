#[allow(dead_code)]
pub mod format;
#[allow(dead_code)]
pub mod types;
pub mod error;

use std::fs::File;
use std::io::{Read, BufReader};

use common::Lexer;
use self::format::Shader;
use self::error::{VMTLoadResult, VMTLoadError};

#[derive(Debug)]
pub struct VMTFile<'a> {
    // A paddingless string containing all of the strings in the vmt.
    // Used to create slices for the various shader elements. The reason
    // this exists is to avoid unnecessary memory allocation, which is
    // a very slow process compared to just slicing a string that's
    // allocated just once.
    #[allow(dead_code)]
    vmt_els: String,
    shader: Shader<'a>,
}


impl<'a> VMTFile<'a> {
    pub fn open(file: &mut File) -> VMTLoadResult<VMTFile> {
        let mut buf_read = BufReader::new(file);
        let mut vmt_string = String::new();

        try!(buf_read.read_to_string(&mut vmt_string).map_err(VMTLoadError::Io));
        let lexer = try!(Lexer::new(&vmt_string[..]).map_err(VMTLoadError::VMT));

        // The length of the string that holds all of the vmt strings
        let mut element_len = 0;
        // The number of elements in the vmt string
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


        unsafe {
            Ok(VMTFile{shader: try!(Shader::from_raw_parts(&lexer.tokens, &vmt_elements[..], &element_lens).map_err(VMTLoadError::VMT)),
                       vmt_els: vmt_elements})
        }
    }

    pub fn get_shader(&self) -> &Shader {
        &self.shader
    }
}
