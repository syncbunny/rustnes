use std::rc::Rc;
use std::cell::RefCell;

use crate::ppu::*;
use crate::apu::*;

pub struct MMU {
	mapper: u8,
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
	ppu: Rc<RefCell<PPU>>,
	apu: Rc<RefCell<APU>>
}

impl MMU {
	pub fn new(ppu:Rc<RefCell<PPU>>, apu:Rc<RefCell<APU>>) -> MMU {
		MMU {
			mapper: 0,
			wram: vec![0; 0x0800],
			prom: Vec::new(),
			crom: Vec::new(),
			ppu: ppu,
			apu: apu,
		}
	}

	pub fn read_1byte(&self, addr:u16) -> u8 {
		// TODO: address mapping

		let ret:u8;

		match addr {
			0x0000 ..= 0x07FF => {
				ret = self.wram[addr as usize];
			}
			0x2002 => {
				ret = self.ppu.borrow().get_sr();
			}
			0x8000 ..= 0xFFFF => {
				ret = self.prom[(addr - 0x8000) as usize];	
			}
			_ => {
				panic!("mmu.read_1byte: unmapped address: {:x}", addr);
			}
		}

		println!("read_1byte({:x}) -> {:x}", addr, ret);
		return ret;
	}

	pub fn read_2bytes(&self, addr:u16) -> u16{
		// TODO: address mapping

		let mut ret:u16 = 0;

		if addr >= 0x8000 {
			ret = self.prom[(addr - 0x8000) as usize] as u16;	
			ret |= (self.prom[(addr - 0x8000 + 1) as usize] as u16) << 8;
		}

		println!("read_2bytes({:x}) -> {:x}", addr, ret);
		return ret;
	}

	pub fn indirect_y(&self, addr: u16, y: u8) -> u16 {
		let z = self.read_1byte(addr);

		let mut p:u16 = self.read_1byte(z as u16) as u16;
		p |= (self.read_1byte((z+1) as u16) as u16) << 8;
		p = p.wrapping_add(y as u16);

		return p;
	}

	pub fn write(&mut self, addr:u16, n:u8) {
		// TODO: address mapping

		match addr {
			0x0000 ..= 0x07FF => {
				self.wram[addr as usize] = n;
			}
			0x2000 => {
				self.ppu.borrow_mut().set_cr1(n);
			}
			0x2001 => {
				self.ppu.borrow_mut().set_cr2(n);
			}
			0x4010 => {
				self.apu.borrow_mut().set_dmc1(n);
			}
			0x4015 => {
				self.apu.borrow_mut().set_ch_ctrl(n);
			}
			0x4017 => {
				self.apu.borrow_mut().set_frame_counter(n);
			}
			_ => {
				panic!("mmi.write: unmapped address: {:x}", addr);
			}
		}
		println!("write({:x}, {:x})", addr, n);
	}

	pub fn push_2bytes(&mut self, addr:u16, n:u16) {
		self.write(addr -0, (n >> 8) as u8);
		self.write(addr -1, (n & 0x00FF) as u8);
	}

	pub fn pop_2bytes(&self, addr: u16) -> u16 {
		let mut ret: u16;
		ret = self.read_1byte(addr+1) as u16;
		ret |= (self.read_1byte(addr+2) as u16) << 8;
		return ret;
	}

	pub fn set_mapper(&mut self, m: u8) {
		self.mapper = m;
		println!("prom.mapper={}", self.mapper);
	}

	pub fn set_PROM(&mut self, prom: &[u8]) {
		self.prom = prom.to_vec();
		println!("prom.len={}", self.prom.len());
	}

	pub fn set_CROM(&mut self, crom: &[u8]) {
		self.crom = crom.to_vec();
		println!("crom.len={}", self.crom.len());
	}
}
