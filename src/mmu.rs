use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use crate::ppu::*;
use crate::apu::*;
use crate::pad::*;
use crate::io::*;
use crate::events::*;

pub struct MMU {
	mapper: u8,
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
	ppu: Rc<RefCell<PPU>>,
	apu: Rc<RefCell<APU>>,
	io: Arc<Mutex<IO>>,
	event_queue: Arc<Mutex<EventQueue>>
}

impl MMU {
	pub fn new(
		ppu:Rc<RefCell<PPU>>,
		apu:Rc<RefCell<APU>>,
		io: Arc<Mutex<IO>>,
		event_queue: Arc<Mutex<EventQueue>>
	) -> MMU {
		MMU {
			mapper: 0,
			wram: vec![0; 0x0800],
			prom: Vec::new(),
			crom: Vec::new(),
			ppu: ppu,
			apu: apu,
			io: io,
			event_queue: event_queue
		}
	}

	pub fn read_1byte(&mut self, addr:u16) -> u8 {
		// TODO: address mapping

		let ret:u8;

		match addr {
			0x0000 ..= 0x07FF => {
				ret = self.wram[addr as usize];
			}
			0x2002 => {
				ret = self.ppu.borrow().get_sr();
			}
			0x2007 => {
				ret = self.ppu.borrow_mut().read();
			}
			0x4016 => {
				let mut io = self.io.lock().unwrap();
				ret = io.pad.in1();
			}
			0x4017 => {
				let mut io = self.io.lock().unwrap();
				ret = io.pad.in2();
			}
			0x8000 ..= 0xFFFF => {
				ret = self.prom[(addr - 0x8000) as usize];	
			}
			_ => {
				panic!("mmu.read_1byte: unmapped address: {:x}", addr);
			}
		}

		//println!("read_1byte({:x}) -> {:x}", addr, ret);
		return ret;
	}

	pub fn read_2bytes(&mut self, addr:u16) -> u16{
		// TODO: address mapping

		let mut ret:u16;

		match addr {
			0x0000 ..= 0x07FF => {
				ret = self.wram[(addr as usize)] as u16;
				ret |= (self.wram[(addr + 1) as usize] as u16) << 8;
			}
			0x8000 ..= 0xFFFF => {
				ret = self.prom[(addr - 0x8000) as usize] as u16;
				ret |= (self.prom[(addr - 0x8000 + 1) as usize] as u16) << 8;
			}
			_ => {
				ret = self.read_1byte(addr) as u16;
				ret |= (self.read_1byte(addr + 1) as u16) << 8;
			}
		}

		//println!("read_2bytes({:x}) -> {:x}", addr, ret);
		return ret;
	}

	pub fn indirect(&mut self, addr: u16) -> u16 {
		let addr = self.read_2bytes(addr);

		let mut ret:u16;
		ret = self.read_1byte(addr) as u16;
		let addr = if addr & 0xFF == 0xFF {
			addr & 0xFF00
		} else {
			addr + 1
		};
		ret |= (self.read_1byte(addr) as u16) << 8;

		return ret;
	}

	pub fn indirect_x(&mut self, addr: u16, x: u8) -> u16 {
        let z:u8 = self.read_1byte(addr).wrapping_add(x);
		let mut p:u16 = self.read_1byte(z as u16) as u16;
		let z = z.wrapping_add(1);
		p |= (self.read_1byte(z as u16) as u16) << 8;

        	return p;
	}

	pub fn indirect_y(&mut self, addr: u16, y: u8) -> u16 {
		let z = self.read_1byte(addr);

		let mut p:u16 = self.read_1byte(z as u16) as u16;
		let z = z.wrapping_add(1);
		p |= (self.read_1byte(z as u16) as u16) << 8;
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
			0x2003 => {
				self.ppu.borrow_mut().set_sprite_write_addr(n);
			}
			0x2005 => {
				self.ppu.borrow_mut().set_scroll(n);
			}
			0x2006 => {
				self.ppu.borrow_mut().set_write_addr(n);
			}
			0x2007 => {
				self.ppu.borrow_mut().write(n);
			}
			0x4000 => {
				self.apu.borrow_mut().set_sw1_cr1(n);
			}
			0x4001 => {
				self.apu.borrow_mut().set_sw1_cr2(n);
			}
			0x4002 => {
				self.apu.borrow_mut().set_sw1_fq1(n);
			}
			0x4003 => {
				self.apu.borrow_mut().set_sw1_fq2(n);
			}
			0x4004 => {
				self.apu.borrow_mut().set_sw2_cr1(n);
			}
			0x4005 => {
				self.apu.borrow_mut().set_sw2_cr2(n);
			}
			0x4006 => {
				self.apu.borrow_mut().set_sw2_fq1(n);
			}
			0x4007 => {
				self.apu.borrow_mut().set_sw2_fq2(n);
			}
			0x4008 => {
				self.apu.borrow_mut().set_tw_cr1(n);
			}
			0x400C => {
				self.apu.borrow_mut().set_noise_cr1(n);
			}
			0x4010 => {
				self.apu.borrow_mut().set_dmc1(n);
			}
			0x4011 => {
				self.apu.borrow_mut().set_dmc2(n);
			}
			0x4014 => {
				self.start_dma(n);
			}
			0x4015 => {
				self.apu.borrow_mut().set_ch_ctrl(n);
			}
			0x4016 => {
				let mut io = self.io.lock().unwrap();
				io.pad.out(n);
			}
			0x4017 => {
				self.apu.borrow_mut().set_frame_counter(n);
			}
			_ => {
				panic!("mmi.write: unmapped address: {:x}", addr);
			}
		}
		//println!("write({:x}, {:x})", addr, n);
	}

	pub fn push_2bytes(&mut self, addr:u16, n:u16) {
		self.write(addr -0, (n >> 8) as u8);
		self.write(addr -1, (n & 0x00FF) as u8);
	}

	pub fn pop_2bytes(&mut self, addr: u16) -> u16 {
		let mut ret: u16;
		ret = self.read_1byte(addr+1) as u16;
		ret |= (self.read_1byte(addr+2) as u16) << 8;
		return ret;
	}

	pub fn set_mapper(&mut self, m: u8) {
		self.mapper = m;
		//println!("prom.mapper={}", self.mapper);
	}

	pub fn set_PROM(&mut self, prom: &[u8]) {
		println!("prom.len={}", prom.len());
		self.prom = vec![0; 32768];
		match prom.len() {
			0x4000 => {
				self.prom[16384..32768].copy_from_slice(prom);
			}
			0x8000 => {
				self.prom[0..32768].copy_from_slice(prom);
			}
			_ => {
				panic!("not supported prom size.");
			}
		}
	}

	pub fn set_CROM(&mut self, crom: &[u8]) {
		self.crom = crom.to_vec();
		let mut ppu = self.ppu.borrow_mut();
		ppu.set_crom(crom);
		println!("crom.len={}", self.crom.len());
	}

	fn start_dma(&mut self, n:u8) {
		let src:u16 = (n as u16) << 8;

		let mut ppu = self.ppu.borrow_mut();
		let sprite_mem = ppu.get_sprite_mem();
		sprite_mem[0..256].copy_from_slice(&self.wram[src as usize..src as usize + 256]);

		let mut queue = self.event_queue.lock().unwrap();
		queue.push(Event::new(EventType::DMA));
	}

	pub fn peek_02(&self) -> u8 {
		return self.wram[0x02];
	}

	pub fn peek_03(&self) -> u8 {
		return self.wram[0x03];
	}
}
