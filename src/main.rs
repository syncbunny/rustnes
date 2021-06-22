use std::cell::RefCell;
use std::fs::File;
use std::env;
use std::error::Error;

struct MMU {
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
}

impl MMU {
	fn read16(addr:u16) -> u16{
		// TODO
		return 0x0000;
	}
}

struct CPU {
	a: u8,

	pc: u16,
}

impl CPU {
	fn clock(&self) {
	}

	fn reset(&self) {
	}
}

struct NES {
	cpu: RefCell<CPU>,
	mmu: RefCell<MMU>
}

impl NES {
	fn loadCartridg(&self, path:&str) {
		let file = match File::open(path) {
			Err(e) => panic!("cannot open {}: {}", path, e.description()),
			Ok(f) => f,
		};
	}

	fn clock(&self) {
		let cpu = self.cpu.borrow();
		cpu.clock();
	}

	fn reset(&self) {
		let cpu = self.cpu.borrow();
		cpu.reset();
	}
}

fn main() {
	let args:Vec<String> = env::args().collect();

	let mut mmu = RefCell::new(MMU{
		wram: vec![0; 0x0800],
		prom: Vec::new(),
		crom: Vec::new()
	});
	let mut cpu = RefCell::new(CPU{
		a: 0,
		pc: 0
	});
	let mut nes = NES{
		cpu: cpu,
		mmu: mmu
	};

	nes.loadCartridg(&args[1]);
	nes.reset();
	loop {
		nes.clock();
	}
}
