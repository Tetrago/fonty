use crate::Error;
use crate::Result;
use crate::read;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::io::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Table {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

impl Table {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut tag = [0u8; 4];
        reader.read_exact(&mut tag)?;

        Ok(Self {
            tag,
            checksum: read!(reader => u32)?,
            offset: read!(reader => u32)?,
            length: read!(reader => u32)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Tables {
    tables: HashMap<String, Table>,
}

macro_rules! tables {
    ({ $($name:ident),+ $(,)? }) => {
        $(
            #[allow(dead_code)]
            pub fn $name(&self) -> u32 {
                self.tables[stringify!($name)].offset
            }
        )+
    };
}

impl Tables {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut tables = HashMap::<String, Table>::new();

        reader.seek(SeekFrom::Start(4))?;
        let count = read!(reader => u16)?;

        reader.seek_relative(6)?;

        for _ in 0..count {
            let table = Table::read_from(reader)?;
            tables.insert(
                std::str::from_utf8(&table.tag)
                    .map_err(|_| Error::InvalidSubtable(table.tag))?
                    .to_owned(),
                table,
            );
        }

        Ok(Self { tables })
    }

    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<Table> {
        self.tables.get(name).cloned()
    }

    tables!({
        cmap,
        glyf,
        head,
        hhea,
        hmtx,
        loca,
        maxp,
        name,
        post
    });
}
