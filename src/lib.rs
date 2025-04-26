use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::path::Path;

macro_rules! flag {
    ({ $($variant:ident),+ $(,)? } => ($name:ident, $alias:ident): $repr:ty) => {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $name {
            $($variant),+
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
pub struct Head {
    index_to_loc_format: i16,
}

impl Head {
    fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        reader.seek_relative(46)?;

        Ok(Self {
            index_to_loc_format: read!(reader => i16)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Component {
    pub flags: ComponentFlags,
    pub index: u16,
    pub offset: (i32, i32),
    pub transform: (i16, i16, i16, i16),
}

#[derive(Clone, Debug)]
pub enum Glyph {
    Simple {
        min: (i16, i16),
        max: (i16, i16),
        end_points: Vec<u16>,
        points: Vec<(i16, i16, OutlineFlags)>,
    },
    Compound {
        min: (i16, i16),
        max: (i16, i16),
        components: Vec<Component>,
    },
}

impl Glyph {
    fn read_simple_from<R: Read + Seek>(
        reader: &mut R,
        contours: i16,
        min: (i16, i16),
        max: (i16, i16),
    ) -> io::Result<Self> {
        let mut end_points = Vec::<u16>::with_capacity(contours as usize);

        for _ in 0..contours {
            end_points.push(read!(reader => u16)?);
        }

        let instruction_count = read!(reader => u16)?;
        reader.seek_relative(instruction_count as i64)?;

        let vertex_count = end_points.last().cloned().unwrap_or(0) as usize + 1;
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

        let mut x_pos = 0i16;

        for (x, _, flags) in points.iter_mut() {
            let delta = if OutlineFlag::XshortVector.test(*flags) {
                let value = read!(reader => u8)? as i16;

                if OutlineFlag::XSame.test(*flags) {
                    value
                } else {
                    -value
                }
            } else {
                if OutlineFlag::XSame.test(*flags) {
                    0
                } else {
                    read!(reader => i16)?
                }
            };

            x_pos += delta;
            *x = x_pos;
        }

        let mut y_pos = 0i16;

        for (_, y, flags) in points.iter_mut() {
            let delta = if OutlineFlag::YshortVector.test(*flags) {
                let value = read!(reader => u8)? as i16;

                if OutlineFlag::YSame.test(*flags) {
                    value
                } else {
                    -value
                }
            } else {
                if OutlineFlag::YSame.test(*flags) {
                    0
                } else {
                    read!(reader => i16)?
                }
            };

            y_pos += delta;
            *y = y_pos;
        }

        Ok(Glyph::Simple {
            min,
            max,
            end_points,
            points,
        })
    }

    fn read_compound_from<R: Read + Seek>(
        reader: &mut R,
        min: (i16, i16),
        max: (i16, i16),
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
            let transform = if ComponentFlag::WeHaveATwoByTwo.test(flags) {
                read!(reader => (i16, i16, i16, i16))?
            } else if ComponentFlag::WeHaveAnXAndYScale.test(flags) {
                let (a, b) = read!(reader => (i16, i16))?;
                (a, 0, 0, b)
            } else if ComponentFlag::WeHaveAScale.test(flags) {
                let scale = read!(reader => i16)?;
                (scale, 0, 0, scale)
            } else {
                (1, 0, 0, 1)
            };

            if ComponentFlag::WeHaveInstructions.test(flags) {
                let instructions = read!(reader => u16)?;
                reader.seek_relative(instructions as i64)?;
            }

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

    pub fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let contours = read!(reader => i16)?;
        let min = read!(reader => (i16, i16))?;
        let max = read!(reader => (i16, i16))?;

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
    head: Head,
    glyf_offsets: HashMap<u16, u32>,
    glyphs: HashMap<u16, Glyph>,
}

impl Ttf {
    fn parse_tables<R: Read + Seek>(reader: &mut R) -> Result<HashMap<String, Table>> {
        let mut tables = HashMap::<String, Table>::new();

        reader.seek(SeekFrom::Start(4))?;
        let count = read!(reader => u16)?;

        reader.seek_relative(6)?;

        for _ in 0..count {
            let table = Table::read_from(reader)?;
            tables.insert(std::str::from_utf8(&table.tag)?.to_owned(), table);
        }

        Ok(tables)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let mut reader = File::open(path)?;
        let tables = Self::parse_tables(&mut reader)?;

        reader.seek(SeekFrom::Start(tables["head"].offset as u64))?;
        let head = Head::read_from(&mut reader)?;

        Ok(Self {
            reader,
            tables,
            head,
            glyf_offsets: HashMap::new(),
            glyphs: HashMap::new(),
        })
    }

    fn glyf_offset(&mut self, index: u16) -> Result<u32> {
        if let Some(offset) = self.glyf_offsets.get(&index) {
            Ok(*offset)
        } else {
            let base = self.tables["loca"].offset;
            let row_size = if self.head.index_to_loc_format != 0 {
                2
            } else {
                1
            };

            self.reader
                .seek(SeekFrom::Start(base as u64 + row_size * index as u64))?;

            let offset = self.tables["glyf"].offset as u32
                + if self.head.index_to_loc_format != 0 {
                    read!(&mut self.reader => u32)?
                } else {
                    (read!(&mut self.reader => u16)? as u32) << 1
                };

            self.glyf_offsets.insert(index, offset);
            Ok(offset)
        }
    }

    pub fn glyph_raw(&mut self, index: u16) -> Result<Glyph> {
        if let Some(glyph) = self.glyphs.get(&index) {
            return Ok(glyph.clone());
        }

        let offset = self.glyf_offset(index)?;
        self.reader.seek(SeekFrom::Start(offset as u64))?;

        let glyph = Glyph::read_from(&mut self.reader)?;
        self.glyphs.insert(index, glyph.clone());

        Ok(glyph)
    }
}
