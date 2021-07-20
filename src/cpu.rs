use std::cell::RefCell;
use std::rc::Rc;

use crate::mmu::*;

const RESET_VECTOR: u16 = 0xFFFC;

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

		// read opcode
		let op:u8 = mmu.read_1byte(self.pc);
		self.pc += 1;

		match op {
			_ => {
				panic!("unsupported opcode:{:x}", op);
			}
		}
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
