use std::sync::Arc;
use std::sync::Mutex;
use crate::io::*;

const CLOCKS_PAR_LINE: u32 = 341;
const DRAWABLE_LINES: u32 = 240;
const SCAN_LINES: u32 = 262;

const CR1_NAMETABLE_MASK: u8 = 0x02;

/* Status Register &H2002 */
const FLAG_VBLANK: u8 = 0x80;
const FLAG_SP_HIT: u8 = 0x40;
const SCANLINE_SPLITE_OVER: u8 = 0x20;
const IFLAG_VBLANK: u8 = 0x7F;
const IFLAG_SP_HIT: u8 = 0xBF;

macro_rules! SET_VBLANK {
	($sr: expr) => {
		$sr |= FLAG_VBLANK;
	}
}

macro_rules! CLEAR_VBLANK {
	($sr: expr) => {
		$sr &= IFLAG_VBLANK;
	}
}

macro_rules! get_nametable {
	($cr1: expr) => {
		$cr1 & CR1_NAMETABLE_MASK
	}
}

pub struct PPU {
	cr1: u8,  // Control Register 1
	cr2: u8,  // Control Register 1
	sr: u8,	  // Status Register
	scroll_y: i32,
	scroll_x: i32,

	line: u32,
	line_clock: u32,

	mem: Vec<u8>,

	io: Arc<Mutex<IO>>
}

impl PPU {
	pub fn new(io:Arc<Mutex<IO>>) -> PPU {
		PPU {
			cr1: 0,
			cr2: 0,
			sr: 0,
			scroll_x: 0,
			scroll_y: 0,

			line: 0,
			line_clock: 0,

			mem: vec![0; 0x4000],

			io: io
		}

	}

	pub fn reset(&mut self) {	
		self.line = 0;
		self.line_clock = 0;
	}

	pub fn clock(&mut self) {
		if self.line == 0 && self.line_clock == 0 {
			self.frame_start();
		}

		self.render_bg(self.line_clock, self.line);

		self.line_clock += 1;
		if self.line_clock >= CLOCKS_PAR_LINE {
			println!("PPU: line {}", self.line);
			self.line_clock = 0;
			self.line += 1;
			if self.line == DRAWABLE_LINES {
				self.start_VR();
			}
			if self.line >= SCAN_LINES {
				CLEAR_VBLANK!(self.sr);
				self.line = 0;
				self.frame_end();
			}
		}
	}

	pub fn set_cr1(&mut self, n:u8) {
		self.cr1 = n;
	}

	pub fn set_cr2(&mut self, n:u8) {
		self.cr2 = n;
	}

	pub fn get_sr(&self) -> u8 {
		return self.sr;
	}

	pub fn set_scroll(&mut self, v:u8) {
		// TODO
	}

	pub fn set_write_addr(&mut self, v:u8) {
		// TODO
	}

	pub fn write(&mut self, v:u8) {
		// TODO
	}

	fn start_VR(&mut self) {
		SET_VBLANK!(self.sr);
	}

	fn frame_start(&mut self) {
		println!("PPU: FrameStart");
	}

	fn frame_end(&mut self) {
	}

	fn render_bg(&mut self, x: u32, y: u32) {
		if x >= 256 || y >= 240 {
			return;
		}

		// calc nametable id
		let mut scrollX: i32;
		let mut scrollY: i32;
		let nametable_id = get_nametable!(self.cr1);

		// TODO: add scroll offset

		// calc nametable address
		let nametable_base:[u32;4] = [
			0x2000, 0x2400, 0x2800, 0x2C00
		];
		let u:u32 = (x/8)%32; // [0 .. 32]
		let v:u32 = (y/8)%30; // [0 .. 30]
		let addr:u32 = nametable_base[nametable_id as usize] + v*32 + u;
		let pat = self.mem[addr as usize];

		// TODO: draw pttern, color
		{
			let mut io = self.io.lock().unwrap();
			//io.vram[0] = 0;
			io.draw_pixel(x, y, pat, pat, pat);
		}
	}
}
