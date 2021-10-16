extern crate memmap;

mod mmu;
mod cpu;
mod ppu;
mod apu;
mod nes;
mod renderer;
mod io;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::env;
use std::thread;
use std::time::Duration;

use crate::cpu::*;
use crate::ppu::*;
use crate::apu::*;
use crate::mmu::*;
use crate::nes::*;
use crate::renderer::*;
use crate::io::*;

fn main() {
	let args:Vec<String> = env::args().collect();
	let mut io = Arc::new(Mutex::new(IO::new()));
	let mut renderer = Renderer::new(Arc::clone(&io));

	thread::spawn(move|| {
		let ppu = Rc::new(RefCell::new(PPU::new()));
		let apu = Rc::new(RefCell::new(APU::new()));
		let mmu = Rc::new(RefCell::new(MMU::new(Rc::clone(&ppu), Rc::clone(&apu))));
		let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&mmu))));
		let mut nes = NES::new(Rc::clone(&cpu), Rc::clone(&mmu), Rc::clone(&ppu), Rc::clone(&apu));

		nes.load_cartridge(&args[1]);
		nes.reset();

		loop {
			nes.clock();
		}
	});

	renderer.event_loop();
}
