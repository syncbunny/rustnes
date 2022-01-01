use std::sync::Arc;
use std::sync::Mutex;

use crate::io::*;

pub struct APUTriangle {
	pub val: u8,
	cr1: u8,
	fq1: u8,
	fq2: u8,

	clock: u16,
	clock_div: u16,
	seq: usize,

	io: Arc<Mutex<IO>>,
}

impl APUTriangle {
	pub fn new(io:Arc<Mutex<IO>>) -> APUTriangle {
		APUTriangle {
			val: 0,
			cr1: 0,
			fq1: 0,
			fq2: 0,

			clock: 0,
			clock_div: 0,
			seq: 0,

			io: io
		}
	}

	pub fn clock(&mut self) {
		if self.clock == 0 {
			// TODO: linear counter, length counter
			self.next_seq();
			self.clock = self.clock_div;
			//println!("triangle: clock_div = {}", self.clock_div);
		} else {
			self.clock -= 1;
		}
	}

	pub fn set_cr1(&mut self, v:u8) -> u8 {
		self.cr1 = v;
		return self.cr1;
	}

	pub fn set_fq1(&mut self, v:u8) -> u8 {
		self.fq1 = v;
		self.clock_div = (self.fq2 as u16) << 8;
		self.clock_div |= self.fq1 as u16;
		//self.clock = 0;
		return self.fq1;
	}

	pub fn set_fq2(&mut self, v:u8) -> u8 {
		self.fq2 = v;
		self.clock_div = (self.fq2 as u16) << 8;
		self.clock_div |= self.fq1 as u16;
		//self.clock = 0;
		return self.fq2;
	}

	fn next_seq(&mut self) {
		let lut:[u8;32] = [
			0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88,
			0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
			0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
			0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF	
		];
		self.val = lut[self.seq];
		
		self.seq += 1;
		if self.seq >= 32 {
			self.seq = 0;
		}
	}
}
