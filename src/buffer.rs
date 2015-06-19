use std::io;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::io::Write;

use super::util;
use super::segment;

pub struct Buffer {
	segment : segment::Segment
}

impl Buffer {
	pub fn from_path(p: &Path) -> io::Result<Buffer> {
		let mut v = vec!();
		let mut f = try!(File::open(p));
		try!(f.read_to_end(&mut v)); 
		Ok(Buffer {
			segment: segment::Segment::from_vec(v)
		})
	}

	pub fn save(&self, to: &Path) -> io::Result<()> {
		let f_r = File::create(to);
		f_r.and_then( |mut f|
			self.segment.iter_slices().fold(Ok(()), |res, val| 
				res.and_then( |_| f.write_all(val)
			)
		))
	}

	pub fn new() -> Buffer {
		Buffer { segment: segment::Segment::new() }
	}

	pub fn len(&self) -> usize {
		self.segment.len()
	}

	pub fn iter_range<'a>(&'a self, from : usize, to : usize) -> segment::Items<'a> {
		self.segment.iter_range(from, to)
	}

	pub fn write(&mut self, offset: usize, val: &[u8]) {
		// util::iter_set(self.segment.mut_iter_range(offset, offset+val.len()), val.iter());
		for (s, d) in val.iter().zip(self.segment.mut_iter_range(offset, offset+val.len())) {
			*d = s.clone();
		}
	}

	pub fn read(&self, offset: usize, len: usize) -> Vec<u8> {
		self.segment.iter_range(offset, offset+len).map(|x| *x).collect::<Vec<u8>>()
	}

	pub fn get_byte(&self, offset: usize) -> u8{
		*self.segment.get(offset)
	}

	pub fn set_byte(&mut self, offset: usize, c: u8) {
		*self.segment.get_mut(offset) = c;
	}

	pub fn insert_byte(&mut self, offset: usize, val: u8) {
		self.insert(offset, &[val])
	}

	pub fn insert(&mut self, offset: usize, val: &[u8]) {
		self.segment.insert_slice(offset, val);
	}

	pub fn find_from(&self, offset : usize, needle : &[u8]) -> Option<usize> {
		self.segment.find_slice_from(offset, needle)
	}

	pub fn remove(&mut self, start_offset: usize, end_offset: usize) -> Vec<u8> {
		self.segment.move_out_slice(start_offset, end_offset)
	}
}