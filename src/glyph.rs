use crate::Head;
use crate::Tables;
use crate::read;
use crate::{ComponentFlag, ComponentFlags};
use crate::{OutlineFlag, OutlineFlags};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
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

pub struct GlyphCache<R: Read + Seek> {
    reader: Rc<RefCell<R>>,
    tables: Rc<RefCell<Tables>>,
    head: Head,
    cache: HashMap<u16, Glyph>,
}

impl<R: Read + Seek> GlyphCache<R> {
    pub fn new(reader: Rc<RefCell<R>>, tables: Rc<RefCell<Tables>>, head: Head) -> Self {
        Self {
            reader,
            tables,
            head,
            cache: HashMap::new(),
        }
    }

    fn offset_of(&self, index: u16) -> io::Result<u32> {
        let base = self.tables.borrow().loca();
        let row_size = if self.head.index_to_loc_format != 0 {
            2
        } else {
            1
        };

        let mut reader = self.reader.borrow_mut();

        reader.seek(SeekFrom::Start(base as u64 + row_size * index as u64))?;

        Ok(self.tables.borrow().glyf() as u32
            + if self.head.index_to_loc_format != 0 {
                read!(reader => u32)?
            } else {
                (read!(reader => u16)? as u32) << 1
            })
    }

    pub fn at(&mut self, index: u16) -> io::Result<Glyph> {
        if let Some(glyph) = self.cache.get(&index) {
            return Ok(glyph.clone());
        }

        let offset = self.offset_of(index)?;
        let mut reader = self.reader.borrow_mut();

        reader.seek(SeekFrom::Start(offset as u64))?;

        let glyph = Glyph::read_from(&mut *reader)?;
        self.cache.insert(index, glyph.clone());

        Ok(glyph)
    }
}
