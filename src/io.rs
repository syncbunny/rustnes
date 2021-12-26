use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::pad::*;

const STENCIL_NONE: u8 = 0;
const STENCIL_BACK_SPRITE: u8 = 1;
const STENCIL_BG: u8 = 2;
const STENCIL_FRONT_SPRITE: u8 = 3;

pub struct IO {
	pub vram: Vec<u8>,
	pub stencil: Vec<u8>,
	pub pad: Pad,
}

pub struct VBR {
	pub in_vbr: bool,
	pub frames: u32
}

impl IO {
	pub fn new() -> IO {
		let pad = Arc::new(Mutex::new(Pad::new()));

		IO {
			vram: vec![0; 256*240*3],
			stencil: vec![0; 256*240],
			pad: Pad::new(),
		}
	}

	pub fn clear(&mut self, r:u8, g:u8, b:u8) {
		self.stencil.fill(0);
		for x in (0..256*240) {
			self.vram[x*3 + 0] = r;
			self.vram[x*3 + 1] = g;
			self.vram[x*3 + 2] = b;
		}
	}

	pub fn get_stencil(&self, x: u32, y: u32) -> u8 {
		return self.stencil[(y*256 +x) as usize];
	}

	pub fn draw_line(&mut self, y: u32, line: &[u8]) {
		for x in 0..256 {
			self.vram[((y*256 +x)*3 +0) as usize] = line[(x*4 +0) as usize];
			self.vram[((y*256 +x)*3 +1) as usize] = line[(x*4 +1) as usize];
			self.vram[((y*256 +x)*3 +2) as usize] = line[(x*4 +2) as usize];
			self.stencil[(y*256 + x) as usize] = line[(x*4 +3) as usize];
		}
	}
	
	pub fn draw_back_sprite(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> bool {
		if self.stencil[(y*256 +x) as usize] <= STENCIL_BACK_SPRITE {
			self.vram[((y*256 +x)*3 +0) as usize] = r;
			self.vram[((y*256 +x)*3 +1) as usize] = g;
			self.vram[((y*256 +x)*3 +2) as usize] = b;
			self.stencil[(y*256 + x) as usize] = STENCIL_BACK_SPRITE;
			return true;
		}
		return false;
	}

	pub fn draw_front_sprite(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> bool {
		if self.stencil[(y*256 +x) as usize] <= STENCIL_FRONT_SPRITE {
			self.vram[((y*256 +x)*3 +0) as usize] = r;
			self.vram[((y*256 +x)*3 +1) as usize] = g;
			self.vram[((y*256 +x)*3 +2) as usize] = b;
			self.stencil[(y*256 + x) as usize] = STENCIL_FRONT_SPRITE;
			return true;
		}
		return false;
	}

	pub fn draw_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) -> bool {
		if self.stencil[(y*256 +x) as usize] <= STENCIL_BG {
			self.vram[((y*256 +x)*3 +0) as usize] = r;
			self.vram[((y*256 +x)*3 +1) as usize] = g;
			self.vram[((y*256 +x)*3 +2) as usize] = b;
			self.stencil[(y*256 + x) as usize] = STENCIL_BG;
			return true;
		}
		return false;
	}
}

impl VBR {
	pub fn new() -> VBR {
		VBR {
			in_vbr: false,
			frames: 0
		}
	}
}
