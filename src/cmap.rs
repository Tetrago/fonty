use std::io;

trait Subtable {
    fn resolve(&mut self, c: char) -> io::Result<u32>;
}

struct Format4;

impl Subtable for Format4 {
    fn resolve(&mut self, c: char) -> io::Result<u32> {
        Ok(0)
    }
}
