use half::f16;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;

macro_rules! flag {
    ({ $($variant:ident),* $(,)? } => ($name:ident, $alias:ident): $repr:ty) => {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $name {
            $($variant),*
        }

        #[allow(dead_code)]
        impl $name {
            fn pos(self) -> $repr {
                self as $repr
            }

            fn mask(self) -> $repr {
                1 << self.pos()
            }

            fn test(self, value: $repr) -> bool {
                (value & self.mask()) != 0
            }
        }

        #[allow(dead_code)]
        type $alias = $repr;
    };
}

macro_rules! read {
    ($reader:expr => ($($type:ty),+)) => {
        (|| {
            let result: io::Result<($($type),+)> = Ok(($(read!($reader => $type)?),+));
            result
        })()
    };
    ($reader:expr => f16) => {{
        let mut buf = [0u8; 2];
        $reader
            .read_exact(&mut buf)
            .map(|_| f16::from_bits(u16::from_be_bytes(buf)))
    }};
    ($reader:expr => $type:ty) => {{
        let mut buf = [0u8; std::mem::size_of::<$type>()];
        $reader
            .read_exact(&mut buf)
            .map(|_| <$type>::from_be_bytes(buf))
    }};
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

flag!({
    OnCurve,
    XshortVector,
    YshortVector,
    Repeat,
    XSame,
    YSame,
} => (OutlineFlag, OutlineFlags): u8);

flag!({
    Arg1And2AreWords,
    ArgsAreXyValues,
    RoundXyToGrid,
    WeHaveAScale,
    Obsolete,
    MoreComponents,
    WeHaveAnXAndYScale,
    WeHaveATwoByTwo,
    WeHaveInstructions,
    UseMyMetrics,
    OverlapCompound,
} => (ComponentFlag, ComponentFlags): u16);

#[derive(Clone, Debug)]
pub struct Component {
    pub flags: ComponentFlags,
    pub index: u16,
    pub offset: (i32, i32),
    pub transform: (f16, f16, f16, f16),
}

#[derive(Clone, Debug)]
pub enum Glyph {
    Simple {
        min: (f16, f16),
        max: (f16, f16),
        indices: Vec<u16>,
        points: Vec<(i16, i16, OutlineFlags)>,
    },
    Compound {
        min: (f16, f16),
        max: (f16, f16),
        components: Vec<Component>,
    },
}

impl Glyph {
    fn read_simple_from<R: Read>(
        reader: &mut R,
        contours: i16,
        min: (f16, f16),
        max: (f16, f16),
    ) -> io::Result<Self> {
        let mut indices = Vec::<u16>::new();
        let mut vertex_count = 0usize;

        for _ in 0..contours {
            let index = read!(reader => u16)?;
            vertex_count = vertex_count.max(index as usize + 1);

            indices.push(index);
        }

        let mut points = Vec::<(i16, i16, OutlineFlags)>::with_capacity(vertex_count);

        while points.len() < vertex_count {
            let flags = read!(reader => OutlineFlags)?;
            points.push((0, 0, flags));

            if OutlineFlag::Repeat.test(flags) {
                let repetitions = read!(reader => u8)?;

                for _ in 0..repetitions {
                    points.push((0, 0, flags));
                }
            }
        }

        for (x, _, flags) in points.iter_mut() {
            if OutlineFlag::XshortVector.test(*flags) {
                *x = read!(reader => u8)? as i16;
            } else {
                *x = read!(reader => i16)?;
            }
        }

        for (_, y, flags) in points.iter_mut() {
            if OutlineFlag::YshortVector.test(*flags) {
                *y = read!(reader => u8)? as i16;
            } else {
                *y = read!(reader => i16)?;
            }
        }

        Ok(Glyph::Simple {
            min,
            max,
            indices,
            points,
        })
    }

    fn read_compound_from<R: Read>(
        reader: &mut R,
        min: (f16, f16),
        max: (f16, f16),
    ) -> io::Result<Self> {
        let mut flags = ComponentFlag::MoreComponents.mask();
        let mut components = Vec::<Component>::new();

        while ComponentFlag::MoreComponents.test(flags) {
            flags = read!(reader => ComponentFlags)?;

            let index = read!(reader => u16)?;

            let offset = if ComponentFlag::Arg1And2AreWords.test(flags) {
                if ComponentFlag::ArgsAreXyValues.test(flags) {
                    let (x, y) = read!(reader => (i16, i16))?;
                    (x as i32, y as i32)
                } else {
                    let (x, y) = read!(reader => (u16, u16))?;
                    (x as i32, y as i32)
                }
            } else {
                if ComponentFlag::ArgsAreXyValues.test(flags) {
                    let (x, y) = read!(reader => (i8, i8))?;
                    (x as i32, y as i32)
                } else {
                    let (x, y) = read!(reader => (u8, u8))?;
                    (x as i32, y as i32)
                }
            };

            let transform = read!(reader => (f16, f16, f16, f16))?;

            components.push(Component {
                flags,
                index,
                offset,
                transform,
            });
        }

        Ok(Glyph::Compound {
            min,
            max,
            components,
        })
    }

    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
        let contours = read!(reader => i16)?;
        let min = read!(reader => (f16, f16))?;
        let max = read!(reader => (f16, f16))?;

        if contours >= 0 {
            Self::read_simple_from(reader, contours, min, max)
        } else {
            Self::read_compound_from(reader, min, max)
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
struct Table {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: u32,
    pub length: u32,
}

impl Table {
    pub fn read_from<R: Read>(reader: &mut R) -> io::Result<Self> {
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

pub struct Ttf {
    reader: File,
    tables: HashMap<String, Table>,
    glyf_offsets: Vec<u32>,
    glyphs: HashMap<u16, Glyph>,
}

impl Ttf {
    pub fn open(path: &Path) -> Result<Self> {
        let mut obj = Self {
            reader: File::open(path)?,
            tables: HashMap::new(),
            glyf_offsets: Vec::new(),
            glyphs: HashMap::new(),
        };

        obj.parse_tables()?;
        obj.glyf_offsets.push(obj.tables["glyf"].offset);

        Ok(obj)
    }

    fn parse_tables(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(4))?;
        let count = read!(&mut self.reader => u16)?;

        self.reader.seek_relative(6)?;

        for _ in 0..count {
            let table = Table::read_from(&mut self.reader)?;
            self.tables
                .insert(std::str::from_utf8(&table.tag)?.to_owned(), table);
        }

        Ok(())
    }

    fn get_glyf_offset(&mut self, index: u16) -> Result<u32> {
        if let Some(offset) = self.glyf_offsets.get(index as usize) {
            return Ok(*offset);
        }

        let offset = self.glyf_offsets.last().unwrap();
        self.reader.seek(SeekFrom::Start(*offset as u64))?;

        let skip = index as u32 - self.glyf_offsets.len() as u32;

        for _ in 0..skip {
            Glyph::read_from(&mut self.reader)?;

            self.glyf_offsets
                .push(self.reader.stream_position()? as u32);
        }

        Ok(self.glyf_offsets.last().unwrap().clone())
    }

    pub fn glyph(&mut self, index: u16) -> Result<Glyph> {
        if let Some(glyph) = self.glyphs.get(&index) {
            return Ok(glyph.clone());
        }

        let offset = self.get_glyf_offset(index)?;
        self.reader.seek(SeekFrom::Start(offset as u64))?;

        let glyph = Glyph::read_from(&mut self.reader)?;
        self.glyphs.insert(index, glyph.clone());

        Ok(glyph)
    }
}
