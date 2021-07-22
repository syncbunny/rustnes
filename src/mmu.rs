use std::rc::Rc;
use std::cell::RefCell;

use crate::ppu::*;

pub struct MMU {
	mapper: u8,
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
	ppu: Rc<RefCell<PPU>>
}

impl MMU {
	pub fn new(ppu:Rc<RefCell<PPU>>) -> MMU {
		MMU {
			mapper: 0,
			wram: vec![0; 0x0800],
			prom: Vec::new(),
			crom: Vec::new(),
			ppu: ppu
		}
	}

	pub fn read_1byte(&self, addr:u16) -> u8 {
		// TODO: address mapping

		let mut ret:u8;

		match addr {
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
			_ => {
				panic!("mmi.write: unmapped address: {:x}", addr);
			}
		}
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
