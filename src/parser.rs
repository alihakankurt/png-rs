use std::{
    fs::File,
    io::{Read, Result},
};

pub struct Parser {
    file: File,
}

impl Parser {
    const SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    pub fn new(filepath: &String) -> Option<Self> {
        let file = File::open(filepath);
        if let Err(e) = file {
            println!("Unable to read {}: {}", filepath, e);
            return None;
        }

        return Some(Self {
            file: file.unwrap(),
        });
    }

    pub fn parse(&mut self) -> bool {
        let validation = self.validate_png_signature();
        if let Err(e) = validation {
            println!("Unable to validate png signature: {}", e);
            return false;
        }

        return validation.unwrap();
    }

    fn validate_png_signature(&mut self) -> Result<bool> {
        let mut signature = [0; 8];
        self.file.read_exact(&mut signature)?;
        return Ok(signature == Parser::SIGNATURE);
    }
}
