use std::sync::Arc;
use std::sync::Mutex;
use crate::io::*;
use std::{thread, time};
use crate::events::*;

const CLOCKS_PAR_LINE: u32 = 341;
const DRAWABLE_LINES: u32 = 240;
const SCAN_LINES: u32 = 262;

/* Control Regster1 &H2000 */
const FLAG_NMI_ON_VB: u8 = 0x80;
const CR1_BG_PATTABLE_MASK: u8 = 0x10; // 0: 0x0000, 1:0x1000
const CR1_SP_PATTABLE_MASK: u8 = 0x08; // 0: 0x0000, 1:0x1000
const FLAG_ADDR_INC: u8 = 0x04; // 0: +1, 1: +32
const CR1_NAMETABLE_MASK: u8 = 0x03;

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

macro_rules! get_bg_pattern_table_addr {
	($cr1: expr) => {
		if ($cr1 & CR1_BG_PATTABLE_MASK) == 0 {
			0x0000
		} else {
			0x1000
		}
	}
}

macro_rules! get_sprite_pattern_table_addr {
	($cr1: expr) => {
		if ($cr1 & CR1_SP_PATTABLE_MASK) == 0 {
			0x0000
		} else {
			0x1000
		}
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

	write_mode: u8, // 0 or 1
	write_addr: u16,
	sprite_write_addr: usize,
	read_buffer: u8,
	mem: Vec<u8>,
	sprite_mem: Vec<u8>,

	pattern_lut: Vec<u8>,

	io: Arc<Mutex<IO>>,
	event_queue: Arc<Mutex<EventQueue>>
}

impl PPU {
	pub fn new(io:Arc<Mutex<IO>>, event_queue: Arc<Mutex<EventQueue>>) -> PPU {
		let mut ppu = PPU {
			cr1: 0,
			cr2: 0,
			sr: 0,
			scroll_x: 0,
			scroll_y: 0,

			line: 0,
			line_clock: 0,

			write_mode: 0,
			write_addr: 0,
			sprite_write_addr: 0,
			read_buffer: 0,
			mem: vec![0; 0x4000],
			sprite_mem: vec![0; 256],

			pattern_lut: vec![0; 256*256*8], // Hi * Lo * x

			io: io,
			event_queue: event_queue
		};
		ppu.generate_lut();

		return ppu;
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
			//println!("PPU: line {}", self.line);
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

	// Mapping to 0x2000
	pub fn set_cr1(&mut self, n:u8) {
		self.cr1 = n;
	}

	// Mapping to 0x2001
	pub fn set_cr2(&mut self, n:u8) {
		self.cr2 = n;
	}

	// Mapping to 0x2003
	pub fn set_sprite_write_addr(&mut self, n: u8) {
		self.sprite_write_addr = n as usize;
	}

	pub fn get_sr(&self) -> u8 {
		return self.sr;
	}

	pub fn set_scroll(&mut self, v:u8) {
		// TODO
	}

	pub fn set_write_addr(&mut self, v:u8) {
		if self.write_mode == 0 {
			self.write_addr &= 0x00FF;
			self.write_addr |= (v as u16) << 8;

			self.write_mode = 1;
		} else {
			self.write_addr &= 0xFF00;
			self.write_addr |= (v as u16);

			self.write_mode = 0;
		}
	}

	// Mapping to 0x2007
	pub fn write(&mut self, v:u8) {
		let addr = self.write_addr & 0x3FFF;
		let mut v = v;

		// background pallet or sprite pallet
		if addr >= 0x3F00 && addr <= 0x3FFF {
			v &= 0x3F;
		}

		self.mem[addr as usize] = v;

		// Increment write address
		if self.cr1 & FLAG_ADDR_INC == 0 {
			self.write_addr += 1;
		} else {
			self.write_addr += 32;
		}
	}

	// Mapping to 0x2007
	pub fn read(&mut self) -> u8 {
		let ret:u8;
		match self.write_addr {
			0..=0x3EFF => {
				ret = self.read_buffer;
				self.read_buffer = self.mem[self.write_addr as usize];
			}
			0x3F00..=0x3FFF => {
				ret = self.mem[self.write_addr as usize];
			}
			_ => {
				panic!("ppu.read: unmapped address: {:x}", self.write_addr);
			}
		}

		// Increment write address
		if self.cr1 & FLAG_ADDR_INC == 0 {
			self.write_addr += 1;
		} else {
			self.write_addr += 32;
		}

		return ret;
	}

	fn start_VR(&mut self) {
		SET_VBLANK!(self.sr);
		if (self.cr1 & FLAG_NMI_ON_VB) != 0 {
			let mut queue = self.event_queue.lock().unwrap();
			queue.push(Event::new(EventType::NMI));			
		}
	}

	fn frame_start(&mut self) {
		//println!("PPU: FrameStart");
		//let sleep_dur = time::Duration::from_millis(3000);
		//thread::sleep(sleep_dur);
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
		//  +-----------+-----------+
		//  | 2 ($2800) | 3 ($2C00) |
		//  +-----------+-----------+
		//  | 0 ($2000) | 1 ($2400) |
		//  +-----------+-----------+
		let nametable_base:[u32;4] = [
			0x2000, 0x2400, 0x2800, 0x2C00
		];
		let u:u32 = (x/8)%32; // [0 .. 32]
		let v:u32 = (y/8)%30; // [0 .. 30]
		let addr:u32 = nametable_base[nametable_id as usize] + v*32 + u;
		let pat_id:u8 = self.mem[addr as usize]; // pattern id [0..255]

		let pat_base:u16 = get_bg_pattern_table_addr!(self.cr1);
		let pat_addr:u16 = pat_base + ((pat_id as u16)<< 4);
		let pat_addr_lo:u16 = pat_addr + (y%8) as u16;
		let pat_addr_hi:u16 = pat_addr_lo + 8;
		let pat_lo = self.mem[pat_addr_lo as usize];
		let pat_hi = self.mem[pat_addr_hi as usize];
		let pat = self.pattern_lut[(((pat_hi as usize)*256 + (pat_lo as usize))*8 + (x as usize)%8) as usize];

		// TODO: draw pttern, color
		{
			let mut io = self.io.lock().unwrap();
			let pat:u8 = pat << 6;
			io.draw_pixel(x, y, pat, pat, pat);
		}
	}

	pub fn get_sprite_mem(&mut self) -> &mut Vec<u8> {
		&mut self.sprite_mem
	}

    pub fn set_crom(&mut self, crom: &[u8]) {
		self.mem[0..0x2000].copy_from_slice(crom);
	}

	fn generate_lut(&mut self) {
		for hi in 0..=255 {
			for lo in 0..=255 {
				for x in 0..8 {
					let pat_hi:u8 = (hi >> (7-x)) & 0x01;
					let pat_lo:u8 = (lo >> (7-x)) & 0x01;
					let pat:u8 = pat_hi << 1 | pat_lo;
					//println!("LUT({:08b}, {:08b}, {}) -> {:02b}", hi, lo, x, pat);

					let mut index: usize = hi as usize;
					index *= 256;
					index += lo as usize;
					index *= 8;
					index += x as usize;

					self.pattern_lut[index] = pat;
				}
			}
		}
	}
}
