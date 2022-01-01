use std::sync::Arc;
use std::sync::Mutex;

use crate::io::*;
use crate::apu_frame::*;
use crate::apu_triangle::*;

const CLOCK_DIV_FRAME: i32 = 7457;
const CLOCK_FQ: u32 = 1789772; // NTSC
const RENDER_FQ: u32 = 44100;

pub struct APU {
	sw1c1: u8,         // 0x4000
	sw1c2: u8,         // 0x4001
	sw1fq1: u8,        // 0x4002
	sw1fq2: u8,        // 0x4003
	sw2c1: u8,         // 0x4004
	sw2c2: u8,         // 0x4005
	sw2fq1: u8,        // 0x4006
	sw2fq2: u8,        // 0x4007
	twc: u8,           // 0x4008
	twfq1: u8,         // 0x400A
	twfq2: u8,         // 0x400B
	nc: u8,            // 0x400C
	nfq1: u8,          // 0x400E
	nfq2: u8,          // 0x400F
	dmc1: u8,          // 0x4010
	dmc2: u8,          // 0x4011
	dmc3: u8,          // 0x4012
	dmc4: u8,          // 0x4013
	ch_ctrl: u8,       // 0x4015
	frame_counter: u8, // 0x4017

	clock_frame: i32,

	frame: APUFrame,
	triangle: APUTriangle,

	render_clock: u32,
	stall: bool,
	io: Arc<Mutex<IO>>
}

impl APU {
	pub fn new(io:Arc<Mutex<IO>>) -> APU {
		APU {
			sw1c1: 0,
			sw1c2: 0,
			sw1fq1: 0,
			sw1fq2: 0,
			sw2c1: 0,
			sw2c2: 0,
			sw2fq1: 0,
			sw2fq2: 0,
			twc: 0,
			twfq1: 0,
			twfq2: 0,
			nc: 0,
			nfq1: 0,
			nfq2: 0,
			dmc1: 0,
			dmc2: 0,
			dmc3: 0,
			dmc4: 0,
			ch_ctrl: 0,
			frame_counter: 0,

			clock_frame:0,

			frame: APUFrame::new(),
			triangle: APUTriangle::new(Arc::clone(&io)),

			render_clock: 0,
			stall: false,
			io: io
		}
	}

	pub fn reset(&mut self) {
		// TODO
	}

	pub fn clock(&mut self) {
		if !self.stall {
			self.triangle.clock();
			{
				if self.clock_frame <= 0 {
					self.frame.clock();
					self.clock_frame = CLOCK_DIV_FRAME -1;
				} else {
					self.clock_frame -= 1;
				}
			}
		}

		if self.render_clock == 0 {
			let mut io = self.io.lock().unwrap();
			if io.write_audio(self.triangle.val) {
				self.stall = false;
			} else {
				self.stall = true;
			}
			self.render_clock = CLOCK_FQ/RENDER_FQ -1;
		} else {
			self.render_clock -= 1;
		}
		// TODO
	}

	pub fn set_sw1_cr1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw1_cr2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw1_fq1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw1_fq2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw2_cr1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw2_cr2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw2_fq1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_sw2_fq2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_dmc1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_dmc2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_ch_ctrl(&mut self, v: u8) {
		// TODO
	}

	pub fn set_tw_cr1(&mut self, v: u8) {
		self.twc = self.triangle.set_cr1(v);
	}

	pub fn set_tw_cr2(&mut self, _v: u8) {
		// unused io
	}

	pub fn set_tw_fq1(&mut self, v: u8) {
		self.twfq1 = self.triangle.set_fq1(v);
	}

	pub fn set_tw_fq2(&mut self, v: u8) {
		self.twfq2 = self.triangle.set_fq2(v);
	}

	pub fn set_noise_cr1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_noise_cr2(&mut self, v: u8) {
		// unused
	}

	pub fn set_noise_fq1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_noise_fq2(&mut self, v: u8) {
		// TODO
	}

	pub fn set_frame_counter(&mut self, v: u8) {
		// TODO
	}
}
