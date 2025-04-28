use crate::Cmap;
use crate::Glyph;
use crate::GlyphCache;
use crate::Head;
use crate::Result;
use crate::Tables;
use std::cell::RefCell;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;
use std::pin::Pin;

pub trait Ttf {
    fn glyph(&mut self, c: char) -> Result<Glyph>;
    fn glyph_at(&mut self, index: u16) -> Result<Glyph>;
}

pub struct AbstractTtf<'a, R: Read + Seek> {
    cmap: Cmap<'a, R>,
    glyph_cache: GlyphCache<'a, R>,
}

impl<'a, R: Read + Seek> AbstractTtf<'a, R> {
    pub fn new(reader: &'a RefCell<R>) -> Result<Self> {
        let tables = Tables::read_from(&mut *reader.borrow_mut())?;

        reader
            .borrow_mut()
            .seek(SeekFrom::Start(tables.head() as u64))?;
        let head = Head::read_from(&mut *reader.borrow_mut())?;

        Ok(Self {
            cmap: { Cmap::new(reader, &tables)? },
            glyph_cache: GlyphCache::new(reader, &tables, head),
        })
    }
}

impl<'a, R: Read + Seek> Ttf for AbstractTtf<'a, R> {
    fn glyph(&mut self, c: char) -> Result<Glyph> {
        self.glyph_cache.at(self.cmap.resolve(c)?)
    }

    fn glyph_at(&mut self, index: u16) -> Result<Glyph> {
        self.glyph_cache.at(index)
    }
}

pub struct OwnedTtf<'a, R: Read + Seek> {
    ttf: AbstractTtf<'a, R>,
    _reader: Pin<Box<RefCell<R>>>,
}

impl<'a, R: Read + Seek> OwnedTtf<'a, R> {
    pub fn new(reader: R) -> Result<Self> {
        let reader = Box::pin(RefCell::new(reader));

        let ttf = AbstractTtf::new(unsafe { &*(reader.as_ref().get_ref() as *const RefCell<_>) })?;

        Ok(OwnedTtf {
            ttf,
            _reader: reader,
        })
    }
}

impl<'a, R: Read + Seek> Ttf for OwnedTtf<'a, R> {
    fn glyph(&mut self, c: char) -> Result<Glyph> {
        self.ttf.glyph(c)
    }

    fn glyph_at(&mut self, index: u16) -> Result<Glyph> {
        self.ttf.glyph_at(index)
    }
}

pub fn open(path: &Path) -> Result<OwnedTtf<File>> {
    let file = File::open(path)?;
    OwnedTtf::new(file)
}
