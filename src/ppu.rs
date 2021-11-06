const CLOCKS_PAR_LINE: i32 = 341;
const DRAWABLE_LINES: i32 = 240;
const SCAN_LINES: i32 = 262;

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

pub struct PPU {
	cr1: u8,  // Control Register 1
	cr2: u8,  // Control Register 1
	sr: u8,	  // Status Register

	line: i32,
	line_clock: i32
}

impl PPU {
	pub fn new() -> PPU {
		PPU {
			cr1: 0,
			cr2: 0,
			sr: 0,

			line: 0,
			line_clock: 0
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

		if (self.line_clock == 0) {
			self.render_bg(self.line);
		}

		//self.line_clock += 1;
		self.line_clock += CLOCKS_PAR_LINE;
		if self.line_clock >= CLOCKS_PAR_LINE {
			println!("PPU: line {}", self.line);
			self.line_clock -= CLOCKS_PAR_LINE;
			self.line += 1;
			if self.line == DRAWABLE_LINES {
				self.start_VR();
			}
			if self.line >= SCAN_LINES {
				CLEAR_VBLANK!(self.sr);
				self.line -= SCAN_LINES;
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

	fn render_bg(&mut self, y: i32) {
	}
}
