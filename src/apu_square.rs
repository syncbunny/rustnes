use crate::apu::LENGTH_COUNTER_LUT;
use crate::apu::U4_2_F32_LUT;
use crate::apu_envelope::*;

pub struct APUSquare{
	pub val: f32,
	n: u8,
	cr1: u8,
	cr2: u8,
	fq1: u8,
	fq2: u8,

	length_counter: u8,

	clock: u16,
	clock_div: u16,

	seq: usize,

	envelope: APUEnvelope,
}

const LENGTH_COUNTER_OFF_MASK:u8 = 0x20;
const DUTY_MASK:u8 = 0xC0;
const DUTY_1_8:u8 = 0x00; // 12.5%
const DUTY_1_4:u8 = 0x40; // 25.0%
const DUTY_1_2:u8 = 0x80; // 25.0%
const DUTY_3_4:u8 = 0xC0; // 75.0%
const DUTY_1_8_VAL:[u8;8] = [0, 1, 0, 0, 0, 0, 0, 0];
const DUTY_1_4_VAL:[u8;8] = [0, 1, 1, 0, 0, 0, 0, 0];
const DUTY_1_2_VAL:[u8;8] = [0, 1, 1, 1, 1, 0, 0, 0];
const DUTY_3_4_VAL:[u8;8] = [0, 1, 1, 1, 1, 1, 1, 0];

impl APUSquare {
	pub fn new(n:u8) -> APUSquare {
		APUSquare {
			val: 0.0,
			n: n,
			cr1: 0,
			cr2: 0,
			fq1: 0,
			fq2: 0,

			length_counter: 0,

			clock: 0,
			clock_div: 0,

			seq: 0,

			envelope: APUEnvelope::new(),
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
		if (self.cr1 & LENGTH_COUNTER_OFF_MASK == 0) && self.length_counter > 0 {
			self.length_counter -= 1;
		}
	}

	pub fn envelope_clock(&mut self) {
		self.envelope.clock();
	}

	pub fn set_cr1(&mut self, v:u8) -> u8 {
		self.cr1 = v;
		self.envelope.set_cr(v);
		return self.cr1;
	}

	pub fn set_cr2(&mut self, v:u8) -> u8 {
		self.cr2 = v;
		return self.cr2;
	}

	pub fn set_fq1(&mut self, v:u8) -> u8 {
		self.fq1 = v;
/*
		self.clock_div &= 0xFF00;
		self.clock_div |= v as u16;
*/
		self.clock_div = (self.fq2 as u16) & 0x07;
		self.clock_div <<= 8;
		self.clock_div |= self.fq1 as u16;
		return self.fq1;
	}

	pub fn set_fq2(&mut self, v:u8) -> u8 {
		self.fq2 = v;
/*
		self.clock_div &= 0x00FF;
		self.clock_div |= ((v&0x07) as u16) << 8; 
*/
		self.clock_div = (self.fq2 as u16) & 0x07;
		self.clock_div <<= 8;
		self.clock_div |= self.fq1 as u16;
		self.length_counter = LENGTH_COUNTER_LUT[v as usize];
		self.envelope.reset();
		return self.fq2;
	}

	pub fn set_ch_ctrl(&mut self, v:u8) {
		if v == 0 {
			self.length_counter = 0;
		}
	}

	fn next_seq(&mut self) {
		let seq:&[u8;8];
		match self.cr1 & DUTY_MASK {
			DUTY_1_8 => {
				seq = &DUTY_1_8_VAL;
			}
			DUTY_1_4 => {
				seq = &DUTY_1_4_VAL;
			}
			DUTY_1_2 => {
				seq = &DUTY_1_2_VAL;
			}
			DUTY_3_4 => {
				seq = &DUTY_3_4_VAL;
			}
			_ => {
				return;
			}
		}

		if seq[self.seq] == 1 {
			self.val = U4_2_F32_LUT[self.envelope.val() as usize];
		} else {
			self.val = U4_2_F32_LUT[0];
		}

		self.seq += 1;
		if self.seq >= 8 {
			self.seq = 0;
		}
	}
}
