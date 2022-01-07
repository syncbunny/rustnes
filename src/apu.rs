use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::io::*;
use crate::apu_frame::*;
use crate::apu_triangle::*;
use crate::apu_noise::*;

const CLOCK_DIV_FRAME: i32 = 7457;
const CLOCK_FQ: u32 = 1789772; // NTSC
const RENDER_FQ: u32 = 44100;

pub static U8_2_F32_LUT: Lazy<Vec<f32>> = Lazy::new(|| generate_u8_2_f32_lut());
pub static U4_2_F32_LUT: Lazy<Vec<f32>> = Lazy::new(|| generate_u4_2_f32_lut());
pub const CH_CTRL_SQUARE_1:u8 = 0x01;
pub const CH_CTRL_SQUARE_2:u8 = 0x02;
pub const CH_CTRL_TRIANGLE:u8 = 0x04;
pub const CH_CTRL_NOISE:u8 = 0x08;
pub const CH_CTRL_DMC:u8 = 0x10;
pub const LENGTH_COUNTER_LUT: [u8;256] = [
	/* 0000 0___ */ 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A,
	/* 0000 1___ */ 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE,
	/* 0001 0___ */ 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14,
	/* 0001 1___ */ 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
	/* 0010 0___ */ 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28,
	/* 0010 1___ */ 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
	/* 0011 0___ */ 0x50, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50,
	/* 0011 1___ */ 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
	/* 0100 0___ */ 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0,
	/* 0100 1___ */ 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
	/* 0101 0___ */ 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C, 0x3C,
	/* 0101 1___ */ 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A,
	/* 0110 0___ */ 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E,
	/* 0110 1___ */ 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
	/* 0111 0___ */ 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A,
	/* 0111 1___ */ 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E, 0x0E,
	/* 1000 0___ */ 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
	/* 1000 1___ */ 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	/* 1001 0___ */ 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	/* 1001 1___ */ 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
	/* 1010 0___ */ 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
	/* 1010 1___ */ 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14,
	/* 1011 0___ */ 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60,
	/* 1011 1___ */ 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16,
	/* 1100 0___ */ 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0,
	/* 1100 1___ */ 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
	/* 1101 0___ */ 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48,
	/* 1101 1___ */ 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A,
	/* 1110 0___ */ 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
	/* 1110 1___ */ 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C,
	/* 1111 0___ */ 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
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
	noise: Rc<RefCell<APUNoise>>,

	render_clock: u32,
	stall: bool,
	io: Arc<Mutex<IO>>
}

impl APU {
	pub fn new(io:Arc<Mutex<IO>>) -> APU {
		let triangle = Rc::new(RefCell::new(APUTriangle::new()));
		let noise = Rc::new(RefCell::new(APUNoise::new()));
	
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

			frame: APUFrame::new(
				Rc::clone(&triangle),
				Rc::clone(&noise)
			),
			triangle: triangle, 
			noise: noise,

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
			self.noise.borrow_mut().clock();
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
			let mut val: f32 = 0.0;
			val += self.triangle.borrow().val;
			val += self.noise.borrow().val;
			if io.write_audio(val) {
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
		self.triangle.borrow_mut().set_ch_ctrl(v);
		self.noise.borrow_mut().set_ch_ctrl(v);
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
		self.nc = self.noise.borrow_mut().set_cr(v);
	}

	pub fn set_noise_cr2(&mut self, _v: u8) {
		// unused io
	}

	pub fn set_noise_fq1(&mut self, v: u8) {
		self.nfq1 = self.noise.borrow_mut().set_fq1(v);
	}

	pub fn set_noise_fq2(&mut self, v: u8) {
		self.nfq2 = self.noise.borrow_mut().set_fq2(v);
	}

	pub fn set_frame_cr(&mut self, v: u8) {
		self.frame_cr = self.frame.set_cr(v);
		self.clock_frame = CLOCK_DIV_FRAME -1;
	}
/*
	fn genrate_lut() {
		for x in 0..256 {
			let a = x as f32/256.0; // [0..1]
			let a = a - 0.5; // [-0.5..0.5]
			//let a = a * 0.5; // [-0.25..0.25]
			self.u8_to_f32_lut[x] = a;
		}
	}
*/
}

fn generate_u8_2_f32_lut() -> Vec<f32> {
	let mut v:Vec<f32> = Vec::with_capacity(256);
	for x in 0..256 {
		let a = x as f32/256.0; // [0..1]
		let a = a - 0.5; // [-0.5..0.5]
		//let a = a * 0.5; // [-0.25..0.25]
		v.push(a);
	}

	return v;
}

fn generate_u4_2_f32_lut() -> Vec<f32> {
	let mut v:Vec<f32> = Vec::with_capacity(256);
	for x in 0..16 {
		let a = x as f32/16.0; // [0..1]
		let a = a - 0.5; // [-0.5..0.5]
		v.push(a);
	}

	return v;
}
