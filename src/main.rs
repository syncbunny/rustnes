extern crate memmap;

mod mmu;
mod cpu;
mod ppu;
mod apu;
mod apu_frame;
mod apu_envelope;
mod apu_triangle;
mod apu_noise;
mod pad;
mod nes;
mod renderer;
mod io;
mod events;
mod ringbuffer;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;
use std::env;
use std::thread;
use std::time::Duration;

use crate::cpu::*;
use crate::ppu::*;
use crate::apu::*;
use crate::pad::*;
use crate::mmu::*;
use crate::nes::*;
use crate::renderer::*;
use crate::io::*;
use crate::events::*;
use crate::ringbuffer::*;

struct Configure {
	cartridge: String,
	use_entry: bool,
	entry: u16,
	nestest: bool,
}

fn main() {
	let mut config = Configure {
		cartridge: "".to_string(),
		use_entry: false,
		entry: 0,
		nestest: false,
	};
	analyze_arg(&mut config);
	if config.cartridge.is_empty() {
		println!("Usage: rustnes [--entry address] cartridge");
		return;
	}

	let mut vbr = Arc::new((Mutex::new(VBR::new()), Condvar::new()));
	let mut io = Arc::new(Mutex::new(IO::new()));
	let mut renderer = Renderer::new(Arc::clone(&io), Arc::clone(&vbr));
	let mut event_queue = Arc::new(Mutex::new(EventQueue::new()));

	thread::spawn(move|| {
		let ppu = Rc::new(RefCell::new(PPU::new(Arc::clone(&io), Arc::clone(&event_queue), Arc::clone(&vbr))));
		let apu = Rc::new(RefCell::new(APU::new(Arc::clone(&io))));
		let pad = Rc::new(RefCell::new(Pad::new()));
		let mmu = Rc::new(RefCell::new(MMU::new(Rc::clone(&ppu), Rc::clone(&apu), Arc::clone(&io), Arc::clone(&event_queue))));
		let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&mmu))));
		let mut nes = NES::new(Rc::clone(&cpu), Rc::clone(&mmu), Rc::clone(&ppu), Rc::clone(&apu), Arc::clone(&event_queue));

		nes.load_cartridge(&config.cartridge);
		if config.use_entry | config.nestest {
			cpu.borrow_mut().set_pc(config.entry);
		} else {
			nes.reset();
		}

		if config.nestest {
			loop {
				nes.clock_nestest();
			}
		} else {
			loop {
				nes.clock();
			}
		}
	});

	renderer.event_loop();
}

fn analyze_arg(config:&mut Configure) {
	let args:Vec<String> = env::args().collect();

	enum Option {
		NONE,
		ENTRY
	}
	let mut cnt = 0;
	let mut option = Option::NONE;
	for arg in args {
		if cnt == 0 {
			cnt += 1;
			continue;
		}

		match &*arg {
			"--entry" => {
				option = Option::ENTRY;
			}
			"--nestest" => {
				config.use_entry = true;
				config.entry = 0xC000;
				config.nestest = true;
				option = Option::NONE;
			}
			_ => {
				match option {
					Option::ENTRY => {
						config.use_entry = true;
						config.entry = u16::from_str_radix(&arg, 16).unwrap();
					}
					Option::NONE => {
						config.cartridge = arg;
					}
				}
				option = Option::NONE;
			}
		}

		cnt += 1;
	}
}
