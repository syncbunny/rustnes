use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use crate::pad::*;
use crate::ringbuffer::*;

const STENCIL_NONE: u8 = 0;
const STENCIL_BACK_SPRITE: u8 = 1;
const STENCIL_BG: u8 = 2;
const STENCIL_FRONT_SPRITE: u8 = 3;
const AUDIO_BUFFER_SIZE: usize = 4096;

pub struct IO {
	pub vram: Vec<u8>,
	pub stencil: Vec<u8>,
	pub audio: RingBuffer<f32>,
	pub pad: Pad,

	wp_audio: usize,
	rp_audio: usize,

	audio_lut: Vec<f32>,
}

pub struct VBR {
	pub in_vbr: bool,
	pub frames: u32
}

impl IO {
	pub fn new() -> IO {
		let pad = Arc::new(Mutex::new(Pad::new()));

		let mut ret = IO {
			vram: vec![0; 256*240*3],
			stencil: vec![0; 256*240],

			audio: RingBuffer::new(AUDIO_BUFFER_SIZE, 0.0),
			wp_audio: 0,
			rp_audio: 0,
			audio_lut: vec![0.0; 256],

			pad: Pad::new(),
		};
		ret.generate_lut();
		
		return ret;
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

	pub fn write_audio(&mut self, v: u8) -> bool {
		return self.audio.write(self.audio_lut[v as usize]);
	}

	pub fn read_audio(&mut self, buf: &mut[f32]) {
		for i in 0..buf.len() {
			match self.audio.read() {
				Some(v) => {buf[i] = v},
				None => {buf[i] = 0.0}
			}
		}
	}

	fn generate_lut(&mut self) {
		for x in 0..256 {
			let a = x as f32/256.0; // [0..1]
			let a = a - 0.5; // [-0.5..0.5]
			//let a = a * 0.5; // [-0.25..0.25]
			self.audio_lut[x] = a;
		}
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
