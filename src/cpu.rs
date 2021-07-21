use std::cell::RefCell; use std::rc::Rc; 
use crate::mmu::*;

const RESET_VECTOR: u16 = 0xFFFC;

const opcode_size: [u16;256] = [
//       0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
/* 00 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 10 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 20 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 30 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 40 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 50 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 60 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 70 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 80 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* 90 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* A0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
/* B0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* C0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* D0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* E0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* F0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub struct CPU {
	a: u8,

	pc: u16,

	clock_remain: u32,
	reset_flag: bool,

	mmu: Rc<RefCell<MMU>>,
}

impl CPU {
	pub fn new(mmu: Rc<RefCell<MMU>>) -> CPU {
		CPU {
			a: 0,
			pc: 0,
			clock_remain: 0,
			reset_flag: false,
			mmu: mmu 
		}
	}

	pub fn clock(&mut self) {
		if self.reset_flag {
			self.do_reset();
		}
	
		if self.clock_remain > 0 {
			self.clock_remain -= 1;
			return;
		}

		let mut mmu = self.mmu.borrow_mut();

		// Addressing modes
		macro_rules! IMM { () => { self.pc } }

		// Oprands
		macro_rules! LDA {
			($ea:expr) => {
				self.a = mmu.read_1byte($ea);
			}
		}

		// read opcode
		let op:u8 = mmu.read_1byte(self.pc);
		self.pc += 1;

		match op {
			0xA9 => { // LDA Immediate
				LDA!( IMM!() );
			}
			_ => {
				panic!("unsupported opcode:{:x}", op);
			}
		}
		self.pc += opcode_size[op as usize];
	}
	
	pub fn reset(&mut self) {
		let mut mmu = self.mmu.borrow_mut();
		self.pc = mmu.read_2bytes(RESET_VECTOR);
		self.reset_flag = true;
		self.clock_remain = 0;
	}
	
	fn do_reset(&mut self) {
		println!("cpu:reset");
		self.reset_flag = false;
		self.clock_remain = 6;
	}
}
