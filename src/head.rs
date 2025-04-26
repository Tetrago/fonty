use crate::read;
use std::io;
use std::io::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Head {
    pub index_to_loc_format: i16,
}

impl Head {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        reader.seek_relative(46)?;

        Ok(Self {
            index_to_loc_format: read!(reader => i16)?,
        })
    }
}
