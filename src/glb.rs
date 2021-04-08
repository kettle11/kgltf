use kjson::*;
use std::io::Read;

use crate::GlTf;

#[derive(Debug, Clone)]

pub struct GLB {
    pub gltf: GlTf,
    pub glb_version: u32,
    // need to include the binary part here as well.
}

#[derive(Debug)]
pub enum GLBError {
    Io(::std::io::Error),
    /// The file's magic number is incorrect. This probably isn't a GLB.
    IncorrectMagicNumber,
    /// The file's formatting is incorrect.
    IncorrectFormatting,
    /// The GLB's inner JSON is incorrectly formatted or could not be parsed.
    InvalidJSON,
}

impl GLB {
    pub fn from_bytes(data: &[u8]) -> Result<Self, GLBError> {
        let reader = std::io::BufReader::new(data);
        Self::from_reader(reader)
    }

    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, GLBError> {
        // Header
        let magic = reader.get_u32()?;
        if magic != 0x46546C67 {
            Err(GLBError::IncorrectMagicNumber)?
        }

        let glb_version = reader.get_u32()?;
        let file_length = reader.get_u32()?;

        // JSON Chunk
        let json_chunk_length = reader.get_u32()?;
        let json_chunk_type = reader.get_u32()?;
        if json_chunk_type != 0x4E4F534A {
            // The chunk type does not match the expected chunk type
            Err(GLBError::IncorrectFormatting)?
        }

        let mut json_string_bytes = vec![0; json_chunk_length as usize];
        reader
            .read_exact(&mut json_string_bytes)
            .map_err(GLBError::Io)?;

        let json_string =
            String::from_utf8(json_string_bytes).map_err(|_| GLBError::IncorrectFormatting)?;
        let gltf = GlTf::from_json(&json_string).ok_or(GLBError::InvalidJSON)?;

        Ok(GLB { gltf, glb_version })
    }
}

trait ReaderExtensions: Read {
    fn get_u32(&mut self) -> Result<u32, GLBError> {
        let mut bytes = [0; 4];
        self.read_exact(&mut bytes).map_err(GLBError::Io)?;
        Ok(u32::from_le_bytes(bytes))
    }
}

impl<R: Read> ReaderExtensions for R {}
