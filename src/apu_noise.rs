use crate::apu::LENGTH_COUNTER_LUT;  
use crate::apu::U4_2_F32_LUT;
use crate::apu::CH_CTRL_NOISE;
use crate::apu_envelope::*;

const RAND_MODE:u8 = 0x80;
const LENGTH_COUNTER_OFF_MASK:u8 = 0x20;

pub struct APUNoise {
	pub val: f32,
	cr: u8,
	fq1: u8,
	fq2: u8,

	length_counter: u8,
	shift_reg: u16,

	clock: u16,
	clock_div: u16,

	envelope: APUEnvelope,
}

impl APUNoise {
	pub fn new() -> APUNoise {
		APUNoise {
			val: 0.0,
			cr: 0,
			fq1: 0,
			fq2: 0,

			length_counter: 0,
			shift_reg: 1,

			clock: 0,
			clock_div: 0,

			envelope: APUEnvelope::new()
		}
	}

	pub fn clock(&mut self) {
		if self.clock == 0 {
			if self.length_counter != 0 {
					self.next_seq();
			}
			self.clock = self.clock_div;
		} else {
			self.clock -= 1;
		}
	}

	pub fn length_clock(&mut self) {
		if (self.cr & LENGTH_COUNTER_OFF_MASK == 0) && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	pub fn envelope_clock(&mut self) {
		self.envelope.clock();
	}

	pub fn set_cr(&mut self, v: u8) -> u8 {
		self.cr = v;
		self.envelope.set_cr(v);
		return self.cr;
	}

	pub fn set_fq1(&mut self, v: u8) -> u8 {
		let lut:[u16; 16] = [
			0x0004, 0x0008, 0x0010, 0x0020,
			0x0040, 0x0060, 0x0080, 0x00A0,
			0x00CA, 0x00FE, 0x017C, 0x01FC,
			0x02FA, 0x03F8, 0x07F2, 0x0FE4,
		];
		self.fq1 = v;

		self.clock_div = lut[(v & 0x0F) as usize];

		return self.fq1;
	}

	pub fn set_fq2(&mut self, v: u8) -> u8 {
		self.fq2 = v;
		self.length_counter = LENGTH_COUNTER_LUT[v as usize];
		self.envelope.reset();
		return self.fq2;
	}

	pub fn set_ch_ctrl(&mut self, v:u8) {
		if v & CH_CTRL_NOISE == 0 {
			self.length_counter = 0;
		}
    }

	fn next_seq(&mut self) {
		let mut exor:bool = false;
		if (self.fq1 & RAND_MODE) == 0 {
			// long mode
			let b0 = self.shift_reg & 0x0001;
			let b1 = (self.shift_reg >> 1) & 0x0001;
			if b0 ^ b1 != 0 {
				exor = true;
			} else {
				exor = false;
			}
		} else {
			// short mode
			let b0 = self.shift_reg & 0x0001;
			let b6 = (self.shift_reg >> 6) & 0x0001;
			if b0 ^ b6 != 0 {
				exor = true;
			} else {
				exor = false;
			}
		}
		self.shift_reg >>= 1;
		if  exor {
			self.shift_reg |= 0x4000;
		}

		if self.shift_reg & 0x0001 != 0 {
			self.val = U4_2_F32_LUT[self.envelope.val() as usize];
		} else {
			self.val = U4_2_F32_LUT[0];
		}
	}	
}
