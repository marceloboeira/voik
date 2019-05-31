
use super::segment::Segment;
use super::position::Position;
use std::io::Error;

pub struct Record<'a> {
    current_offset: usize,

    segment: &'a mut Segment,
}

impl<'a> Record<'a> {
    pub fn new(segment: &'a mut Segment, offset: usize) -> Result<Self, Error>{
        Ok(Self {
            current_offset: offset,
            segment: segment
        })
    }

    pub fn record(&mut self) -> Result<&[u8], Error> {
        self.segment.read_at(self.current_offset)
    }

    pub fn position(&self) -> Position {
        Position::Offset(self.current_offset)
    }

    pub fn record_after(&'a mut self, offset: usize) -> Self {
        Self {
            segment: self.segment,
            current_offset: self.current_offset + offset
        }
    }

    pub fn next(&'a mut self) -> Self {
        self.record_after(1)
    }
}