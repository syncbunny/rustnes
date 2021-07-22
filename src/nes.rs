extern crate memmap;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use memmap::Mmap;

use crate::cpu::*;
use crate::mmu::*;
use crate::ppu::*;

// NES Const
const FLAG6_V_MIRROR: u8           = 0x01;
const FLAG6_HAS_BATTARY_BACKUP: u8 = 0x02;
const FLAG6_HAS_TRAINER: u8        = 0x04;
const FLAG6_HAS_OWN_MIRROR: u8     = 0x80;
const FLAG6_MAPPAER_LOW: u8        = 0xF0;

const FLAG7_VS_UNISYSTEM: u8       = 0x01;
const FLAG7_PLAYCHOICE10: u8       = 0x02;
const FLAG7_NES_2_0: u8            = 0x0C;
const FLAG7_MAPPER_HIGH: u8        = 0xF0;

pub struct NES {
	cpu: Rc<RefCell<CPU>>,
	mmu: Rc<RefCell<MMU>>,
	ppu: Rc<RefCell<PPU>>
}

impl NES {
	pub fn new(cpu: Rc<RefCell<CPU>>, mmu: Rc<RefCell<MMU>>, ppu: Rc<RefCell<PPU>>) -> NES {
		NES {
			cpu: cpu,
			mmu: mmu,
			ppu: ppu
		}
	}

	pub fn load_cartridge(&self, path:&str) {
		let file = match File::open(path) {
			Err(e) => panic!("cannot open {}: {}", path, e.to_string()),
			Ok(f) => f,
		};

		let cartridge = match unsafe { Mmap::map(&file)} {
			Err(e) => panic!("mmap failed.: {}", e.to_string()),
			Ok(m) => m,
		};

		// check header
		if cartridge[0] != 0x4E { panic!("not nes cartridge"); }
		if cartridge[1] != 0x45 { panic!("not nes cartridge"); }
		if cartridge[2] != 0x53 { panic!("not nes cartridge"); }
		if cartridge[3] != 0x1A { panic!("not nes cartridge"); }

		let offset: usize = if (cartridge[5] & FLAG6_HAS_TRAINER) == 0 { 16 } else { 16 + 512 };

		// PROM size & CROM_SIZE
		let prom_size: usize; // [16k]
		let crom_size: usize; // [8k]
		prom_size = cartridge[4] as usize;
		crom_size = cartridge[5] as usize;

		let mut start = offset;
		let mut end = offset + prom_size * 16 * 1024;
		self.mmu.borrow_mut().set_PROM(&cartridge[start .. end]);
		start = end;
		end = start + crom_size *  8 * 1024;
		self.mmu.borrow_mut().set_CROM(&cartridge[start .. end]);

		// Mapper
		let mapper: u8;
		mapper = (cartridge[7] & FLAG7_MAPPER_HIGH) | ((cartridge[6] & FLAG6_MAPPAER_LOW) >> 4);
		self.mmu.borrow_mut().set_mapper(mapper);
	}

	pub fn clock(&self) {
		let mut cpu = self.cpu.borrow_mut();
		cpu.clock();
	}

	pub fn reset(&self) {
		let mut cpu = self.cpu.borrow_mut();
		let mut ppu = self.ppu.borrow_mut();
		cpu.reset();
		ppu.reset();
	}
}
