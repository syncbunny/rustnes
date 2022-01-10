extern crate memmap;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;
use memmap::Mmap;

use crate::cpu::*;
use crate::mmu::*;
use crate::ppu::*;
use crate::apu::*;
use crate::events::*;

// NES Const
const FLAG6_MIRROR: u8             = 0x01;
const FLAG6_HAS_BATTARY_BACKUP: u8 = 0x02;
const FLAG6_HAS_TRAINER: u8        = 0x04;
const FLAG6_HAS_OWN_MIRROR: u8     = 0x80;
const FLAG6_MAPPAER_LOW: u8        = 0xF0;

const FLAG7_VS_UNISYSTEM: u8       = 0x01;
const FLAG7_PLAYCHOICE10: u8       = 0x02;
const FLAG7_NES_2_0: u8            = 0x0C;
const FLAG7_MAPPER_HIGH: u8        = 0xF0;

const CLOCK_DIV_CPU: i32 = 12;
const CLOCK_DIV_PPU: i32 = 4;
const CLOCK_DIV_APU: i32 = 12;

pub struct NES {
	cpu: Rc<RefCell<CPU>>,
	mmu: Rc<RefCell<MMU>>,
	ppu: Rc<RefCell<PPU>>,
	apu: Rc<RefCell<APU>>,

	clock_cpu: i32,
	clock_ppu: i32,
	clock_apu: i32,

	event_queue: Arc<Mutex<EventQueue>>,
}

impl NES {
	pub fn new(cpu: Rc<RefCell<CPU>>, mmu: Rc<RefCell<MMU>>, ppu: Rc<RefCell<PPU>>, apu: Rc<RefCell<APU>>, event_queue: Arc<Mutex<EventQueue>>) -> NES {
		NES {
			cpu: cpu,
			mmu: mmu,
			ppu: ppu,
			apu: apu,
			clock_cpu: 0,
			clock_ppu: 0,
			clock_apu: 0,
			event_queue: event_queue,
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

		// Mirror
		let flag6: u8 = cartridge[6];
		if flag6 & FLAG6_MIRROR == 0 {
			println!("Mirrir Horizontal");
			self.ppu.borrow_mut().set_mirror(Mirror::HORIZONTAL);
		} else {
			println!("Mirrir Vartical");
			self.ppu.borrow_mut().set_mirror(Mirror::VARTICAL);
		}

		// Mapper
		let mapper: u8;
		mapper = (cartridge[7] & FLAG7_MAPPER_HIGH) | ((cartridge[6] & FLAG6_MAPPAER_LOW) >> 4);
		self.mmu.borrow_mut().set_mapper(mapper);
	}

	pub fn nowait(&mut self, b:bool) {
		let mut ppu = self.ppu.borrow_mut();
		ppu.nowait(b);
	}

	pub fn clock(&mut self) {
		//       Master          CPU      PPU    APU
		// NTSC: 21477272.72 Hz  Base/12  Base/4 Base/12

		{
			let mut queue = self.event_queue.lock().unwrap();
			let evt_w = queue.pop();
			match evt_w {
				None => {}
				_ => {
					let evt = evt_w.unwrap();
					match (evt.event_type) {
						EventType::NMI => {
							//println!("NMI!");
							let mut cpu = self.cpu.borrow_mut();
							cpu.nmi();
						}
						EventType::IRQ => {
							let mut cpu = self.cpu.borrow_mut();
							cpu.irq();
						}
						EventType::DMA => {
							//println!("DMA!");
							// Stop CPU 514 cpu-clock
							self.clock_cpu = 514*CLOCK_DIV_CPU;
							//println!("clock_cpu={}", self.clock_cpu);
						}
					}
				}
			}
		}
		{
			let mut ppu = self.ppu.borrow_mut();
			if self.clock_ppu <= 0 {
				ppu.clock();
				self.clock_ppu = CLOCK_DIV_PPU -1;
			} else {
				self.clock_ppu -= 1;
			}
		}

		{
			let mut cpu = self.cpu.borrow_mut();
			if self.clock_cpu <= 0 {
				cpu.clock();
				self.clock_cpu = CLOCK_DIV_CPU -1;
			} else {
				self.clock_cpu -= 1;
			}
		}

		{
			let mut apu = self.apu.borrow_mut();
			if self.clock_apu <= 0 {
				apu.clock();
				self.clock_apu = CLOCK_DIV_APU -1;
			} else {
				self.clock_apu -= 1;
			}
		}
	}

	pub fn clock_nestest(&mut self) {
		if self.clock_cpu <= 0 {
			self.clock();
			let mmu = self.mmu.borrow_mut();
			let m2 = mmu.peek_02();
			let m3 = mmu.peek_03();
			if m2 != 0 || m3 != 0 {
				println!("nestest: {:02X}, {:02X}", m2, m3);
			}
		} else {
			self.clock();
		}
	}

	pub fn reset(&self) {
		let mut cpu = self.cpu.borrow_mut();
		let mut ppu = self.ppu.borrow_mut();
		let mut apu = self.apu.borrow_mut();
		cpu.reset();
		ppu.reset();
		apu.reset();
	}
}
