extern crate memmap;

use std::cell::RefCell;
use std::fs::File;
use std::env;
use std::error::Error;
use memmap::Mmap;

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

// CPU Const
const RESET_VECTOR: u16 = 0xFFFC;

struct MMU {
	mapper: u8,
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
}

impl MMU {
	fn read16(addr:u16) -> u16{
		// TODO
		return 0x0000;
	}

	fn set_mapper(&mut self, m: u8) {
		self.mapper = m;
		println!("prom.mapper={}", self.mapper);
	}

	fn set_PROM(&mut self, prom: &[u8]) {
		self.prom = prom.to_vec();
		println!("prom.len={}", self.prom.len());
	}

	fn set_CROM(&mut self, crom: &[u8]) {
		self.crom = crom.to_vec();
		println!("crom.len={}", self.crom.len());
	}
}

struct CPU {
	a: u8,

	pc: u16,

	clock_remain: u32,
	reset_flag: bool,
}

impl CPU {
	fn clock(&mut self) {
		if (self.reset_flag) {
			self.do_reset();
		}
	}

	fn reset(&mut self) {
		self.reset_flag = true;
		self.clock_remain = 0;
	}

	fn do_reset(&mut self) {
		self.reset_flag = false;
		self.clock_remain = 6;
	}
}

struct NES {
	cpu: RefCell<CPU>,
	mmu: RefCell<MMU>
}

impl NES {

	fn load_cartridge(&self, path:&str) {
		let file = match File::open(path) {
			Err(e) => panic!("cannot open {}: {}", path, e.description()),
			Ok(f) => f,
		};

		let cartridge = match unsafe { Mmap::map(&file)} {
			Err(e) => panic!("mmap failed.: {}", e.description()),
			Ok(m) => m,
		};

		// check header
		if (cartridge[0] != 0x4E) { panic!("not nes cartridge"); }
		if (cartridge[1] != 0x45) { panic!("not nes cartridge"); }
		if (cartridge[2] != 0x53) { panic!("not nes cartridge"); }
		if (cartridge[3] != 0x1A) { panic!("not nes cartridge"); }

		let offset: usize = if ((cartridge[5] & FLAG6_HAS_TRAINER) == 0) { 16 } else { 16 + 512 };

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

	fn clock(&self) {
		let mut cpu = self.cpu.borrow_mut();
		cpu.clock();
	}

	fn reset(&self) {
		let mut cpu = self.cpu.borrow_mut();
		cpu.reset();
	}
}

fn main() {
	let args:Vec<String> = env::args().collect();

	let mut mmu = RefCell::new(MMU{
		mapper: 0,
		wram: vec![0; 0x0800],
		prom: Vec::new(),
		crom: Vec::new()
	});
	let mut cpu = RefCell::new(CPU{
		a: 0,
		pc: 0,
		clock_remain: 0,
		reset_flag: false,
	});
	let mut nes = NES{
		cpu: cpu,
		mmu: mmu
	};

	nes.load_cartridge(&args[1]);
	nes.reset();
	loop {
		nes.clock();
	}
}
