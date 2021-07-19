extern crate memmap;

mod mmu;
mod cpu;
mod nes;

use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use std::error::Error;

use crate::cpu::*;
use crate::mmu::*;
use crate::nes::*;

// CPU Const
const RESET_VECTOR: u16 = 0xFFFC;

fn main() {
	let args:Vec<String> = env::args().collect();

	let mmu = Rc::new(RefCell::new(MMU::new()));
	let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&mmu))));
	let nes = NES::new(Rc::clone(&cpu), Rc::clone(&mmu));

	nes.load_cartridge(&args[1]);
	nes.reset();
	loop {
		nes.clock();
	}
}
