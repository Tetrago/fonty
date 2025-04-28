use crate::Error;
use crate::Result;
use crate::Tables;
use crate::read;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::SeekFrom;
use std::io::prelude::*;
use std::ops::RangeInclusive;

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Format {
    Format4 {
        length: u16,
        language: u16,
        segment_count: u16,
        search_range: u16,
        entry_selector: u16,
        range_shift: u16,
        code_ranges: Vec<RangeInclusive<u16>>,
        id_deltas: Vec<u16>,
        id_range_offsets: Vec<u16>,
        id_range_offsets_offset: u64,
    },
}

impl Format {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let format = read!(reader => u16)?;

        match format {
            4 => {
                let length = read!(reader => u16)?;
                let language = read!(reader => u16)?;
                let segment_count = read!(reader => u16)? >> 1;
                let search_range = read!(reader => u16)?;
                let entry_selector = read!(reader => u16)?;
                let range_shift = read!(reader => u16)?;

                let end_codes = read!(reader => [u16; segment_count as usize])?;
                reader.seek_relative(2)?;
                let start_codes = read!(reader => [u16; segment_count as usize])?;

                let id_deltas = read!(reader => [u16; segment_count as usize])?;

                let id_range_offsets_offset = reader.stream_position()?;
                let id_range_offsets = read!(reader => [u16; segment_count as usize])?;

                let code_ranges: Vec<_> = start_codes
                    .into_iter()
                    .zip(end_codes.into_iter())
                    .map(|(start, end)| start..=end)
                    .collect();

                Ok(Self::Format4 {
                    length,
                    language,
                    segment_count,
                    search_range,
                    entry_selector,
                    range_shift,
                    code_ranges,
                    id_deltas,
                    id_range_offsets,
                    id_range_offsets_offset,
                })
            }
            _ => Err(Error::UnsupportedCmapFormat(format)),
        }
    }

    pub fn resolve<R: Read + Seek>(&self, reader: &mut R, c: char) -> Result<u16> {
        match self {
            Format::Format4 {
                code_ranges,
                id_deltas,
                id_range_offsets,
                id_range_offsets_offset,
                ..
            } => {
                if let Some((segment, range)) = code_ranges
                    .iter()
                    .enumerate()
                    .find(|(_, range)| range.contains(&(c as u16)))
                {
                    if id_range_offsets[segment] == 0 {
                        Ok(id_deltas[segment].wrapping_add(c as u16))
                    } else {
                        let base_offset = id_range_offsets_offset + (segment * 2) as u64;
                        let local_offset =
                            id_range_offsets[segment] + 2 * (c as u16 - range.start());

                        reader.seek(SeekFrom::Start(base_offset + local_offset as u64))?;
                        let index = read!(reader => u16)?;

                        Ok(if index == 0 {
                            index
                        } else {
                            index.wrapping_add(id_deltas[segment])
                        })
                    }
                } else {
                    Err(Error::CodepointNotInCmap(c))
                }
            }
        }
    }
}

pub struct Cmap<'a, R: Read + Seek> {
    reader: &'a RefCell<R>,
    format: Format,
    cache: HashMap<char, u16>,
}

impl<'a, R: Read + Seek> Cmap<'a, R> {
    pub fn new(reader: &'a RefCell<R>, tables: &Tables) -> Result<Self> {
        if let Some(format) = {
            let mut reader = reader.borrow_mut();

            reader.seek(SeekFrom::Start(tables.cmap() as u64 + 2))?;
            let subtable_count = read!(reader => u16)?;

            let mut subtable: Option<Format> = None;

            for _ in 0..subtable_count {
                let platform_id = read!(reader => u16)?;
                reader.seek_relative(2)?;
                let offset = read!(reader => u32)?;

                if platform_id != 0 {
                    continue;
                }

                let prev = reader.stream_position()?;
                reader.seek(SeekFrom::Start(tables.cmap() as u64 + offset as u64))?;

                if let Ok(format) = Format::read_from(&mut *reader) {
                    subtable.replace(format);
                    break;
                }

                reader.seek(SeekFrom::Start(prev))?;
            }

            subtable
        } {
            Ok(Self {
                reader,
                format,
                cache: HashMap::new(),
            })
        } else {
            Err(Error::NoSuitableCmapFormat)
        }
    }

    pub fn resolve(&mut self, c: char) -> Result<u16> {
        if let Some(index) = self.cache.get(&c) {
            Ok(*index)
        } else {
            let index = self.format.resolve(&mut *self.reader.borrow_mut(), c)?;
            self.cache.insert(c, index);
            Ok(index)
        }
    }
}
