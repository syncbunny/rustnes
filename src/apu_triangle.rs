use std::sync::Arc;
use std::sync::Mutex;

use crate::apu::LENGTH_COUNTER_LUT;
use crate::apu::CH_CTRL_TRIANGLE;
use crate::apu::U8_2_F32_LUT;

const LENGTH_COUNTER_OFF_MASK:u8 = 0x80;

pub struct APUTriangle {
	pub val: f32,
	cr1: u8,
	fq1: u8,
	fq2: u8,

	clock: u16,
	clock_div: u16,
	length_counter: u8,
	linear_counter: u8,
	linear_reload: bool,
	seq: usize,
}

impl APUTriangle {
	pub fn new() -> APUTriangle {
		APUTriangle {
			val: 0.0,
			cr1: 0,
			fq1: 0,
			fq2: 0,

			clock: 0,
			clock_div: 0,
			length_counter: 0,
			linear_counter: 0,
			linear_reload: false,
			seq: 0,
		}
	}

	pub fn clock(&mut self) {
		if self.clock == 0 {
			if self.length_counter != 0 && self.linear_counter != 0 {
				self.next_seq();
				self.clock = self.clock_div;
			}			
			self.clock = self.clock_div;
			//println!("triangle: clock_div = {}", self.clock_div);
		} else {
			self.clock -= 1;
		}
	}

	pub fn length_clock(&mut self) {
		if (self.cr1 & LENGTH_COUNTER_OFF_MASK == 0) && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	pub fn linear_clock(&mut self) {
		if self.linear_reload {
			self.linear_counter = self.cr1 & 0x7F;
		} else {
			if self.linear_counter > 0 {
				self.linear_counter -= 1;
			}
		}
		if (self.cr1 & LENGTH_COUNTER_OFF_MASK) == 0 {
			self.linear_reload = false;
		}
	}

	pub fn set_cr1(&mut self, v:u8) -> u8 {
		self.cr1 = v;
		self.linear_counter = self.cr1 & 0x7F;
		return self.cr1;
	}

	pub fn set_fq1(&mut self, v:u8) -> u8 {
		self.fq1 = v;
		self.clock_div = ((self.fq2 & 0x07) as u16) << 8;
		self.clock_div |= self.fq1 as u16;
		//self.clock = 0;
		return self.fq1;
	}

	pub fn set_fq2(&mut self, v:u8) -> u8 {
		self.fq2 = v;
		self.clock_div = ((self.fq2 & 0x07) as u16) << 8;
		self.clock_div |= self.fq1 as u16;
		self.linear_reload = true;
		self.length_counter = LENGTH_COUNTER_LUT[v as usize];
		//self.clock = 0;
		return self.fq2;
	}

	pub fn set_ch_ctrl(&mut self, v:u8) {
		if v & CH_CTRL_TRIANGLE == 0 {
			self.length_counter = 0;
		}
	}

	fn next_seq(&mut self) {
		let lut:[usize;32] = [
			0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88,
			0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
			0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
			0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF	
		];
		self.val = U8_2_F32_LUT[lut[self.seq]];
		
		self.seq += 1;
		if self.seq >= 32 {
			self.seq = 0;
		}
	}
}
