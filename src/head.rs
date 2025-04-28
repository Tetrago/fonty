use crate::Error;
use crate::Result;
use crate::read;
use std::io::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Head {
    pub index_to_loc_format: i16,
}

impl Head {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        reader.seek_relative(50)?;

        let index_to_loc_format = read!(reader => i16)?;
        if index_to_loc_format != index_to_loc_format & 1 {
            return Err(Error::InvalidHeaderField(format!(
                "indexToLocFormat was 0x{:02x}",
                index_to_loc_format
            )));
        }

        Ok(Self {
            index_to_loc_format,
        })
    }
}
