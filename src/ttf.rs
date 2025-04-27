use crate::Cmap;
use crate::Glyph;
use crate::GlyphCache;
use crate::Head;
use crate::Tables;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

pub struct AbstractTtf<R: Read + Seek> {
    cmap: Cmap<R>,
    glyph_cache: GlyphCache<R>,
}

impl<R: Read + Seek> AbstractTtf<R> {
    pub fn new(mut reader: R) -> io::Result<Self> {
        let tables = Rc::new(RefCell::new(Tables::read_from(&mut reader)?));

        reader.seek(SeekFrom::Start(tables.borrow().head() as u64))?;
        let head = Head::read_from(&mut reader)?;

        let reader = Rc::new(RefCell::new(reader));

        Ok(Self {
            cmap: { Cmap::new(reader.clone(), &*tables.borrow())? },
            glyph_cache: GlyphCache::new(reader, tables, head),
        })
    }

    pub fn glyph(&mut self, c: char) -> io::Result<Glyph> {
        self.glyph_cache.at(self.cmap.resolve(c)?)
    }
}

pub type Ttf = AbstractTtf<File>;

pub fn open(path: &Path) -> io::Result<Ttf> {
    let reader = File::open(path)?;
    Ttf::new(reader)
}
