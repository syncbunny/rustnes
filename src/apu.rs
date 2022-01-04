use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::io::*;
use crate::apu_frame::*;
use crate::apu_triangle::*;

const CLOCK_DIV_FRAME: i32 = 7457;
const CLOCK_FQ: u32 = 1789772; // NTSC
const RENDER_FQ: u32 = 44100;

pub const LENGTH_COUNTER_LUT: [u8;256] = [
	/* 0000 0___ */ 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A,
	/* 0001 0___ */ 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14,
	/* 0010 0___ */ 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
	/* 0011 0___ */ 0x50, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50,
	/* 0100 0___ */ 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0,
	/* 0101 0___ */ 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C,
	/* 0110 0___ */ 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E,
	/* 0111 0___ */ 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A,
	/* 1000 0___ */ 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
	/* 1001 0___ */ 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	/* 1010 0___ */ 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
	/* 1011 0___ */ 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60,
	/* 1100 0___ */ 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0,
	/* 1101 0___ */ 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
	/* 1110 0___ */ 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	/* 1111 0___ */ 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
	/* 0000 1___ */ 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
	/* 0001 1___ */ 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
	/* 0010 1___ */ 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
	/* 0011 1___ */ 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
	/* 0100 1___ */ 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
	/* 0101 1___ */ 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A,
	/* 0110 1___ */ 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
	/* 0111 1___ */ 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E,
	/* 1000 1___ */ 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	/* 1001 1___ */ 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
	/* 1010 1___ */ 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14,
	/* 1011 1___ */ 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16,
	/* 1100 1___ */ 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	/* 1101 1___ */ 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A,
	/* 1110 1___ */ 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C,
	/* 1111 1___ */ 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E,
];

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
	frame_cr: u8,      // 0x4017

	clock_frame: i32,

	frame: APUFrame,
	triangle: Rc<RefCell<APUTriangle>>,

	render_clock: u32,
	stall: bool,
	io: Arc<Mutex<IO>>
}

impl APU {
	pub fn new(io:Arc<Mutex<IO>>) -> APU {
		let triangle = Rc::new(RefCell::new(APUTriangle::new(Arc::clone(&io))));
	
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
			frame_cr: 0,

			clock_frame:0,

			frame: APUFrame::new(Rc::clone(&triangle)),
			triangle: triangle, 

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
			self.triangle.borrow_mut().clock();
		}
		{
			if self.clock_frame <= 0 {
				self.frame.clock();
				self.clock_frame = CLOCK_DIV_FRAME -1;
			} else {
				self.clock_frame -= 1;
			}
		}

		if self.render_clock == 0 {
			let mut io = self.io.lock().unwrap();
			if io.write_audio(self.triangle.borrow().val) {
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
		self.twc = self.triangle.borrow_mut().set_cr1(v);
	}

	pub fn set_tw_cr2(&mut self, _v: u8) {
		// unused io
	}

	pub fn set_tw_fq1(&mut self, v: u8) {
		self.twfq1 = self.triangle.borrow_mut().set_fq1(v);
	}

	pub fn set_tw_fq2(&mut self, v: u8) {
		self.twfq2 = self.triangle.borrow_mut().set_fq2(v);
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

	pub fn set_frame_cr(&mut self, v: u8) {
		self.frame_cr = self.frame.set_cr(v);
		self.clock_frame = CLOCK_DIV_FRAME -1;
	}
}
