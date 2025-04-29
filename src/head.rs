use crate::Error;
use crate::Result;
use crate::read;
use std::io::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Head {
    pub units_per_em: u16,
    pub index_to_loc_format: i16,
}

impl Head {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        reader.seek_relative(18)?;
        let units_per_em = read!(reader => u16)?;
        if !(64..=16384).contains(&units_per_em) {
            return Err(Error::InvalidHeaderField("Invalid em scale".into()));
        }

        reader.seek_relative(30)?;
        let index_to_loc_format = read!(reader => i16)?;
        if index_to_loc_format != index_to_loc_format & 1 {
            return Err(Error::InvalidHeaderField(format!(
                "indexToLocFormat was 0x{:02x}",
                index_to_loc_format
            )));
        }

        Ok(Self {
            units_per_em,
            index_to_loc_format,
        })
    }
}
