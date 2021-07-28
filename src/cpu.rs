use std::cell::RefCell; use std::rc::Rc; 
use crate::mmu::*;

const RESET_VECTOR: u16 = 0xFFFC;
const FLG_C: u8 =  0x01;
const FLG_Z: u8 =  0x02;
const FLG_I: u8 =  0x04;
const FLG_D: u8 =  0x08;
const FLG_B: u8 =  0x10;
const FLG_5: u8 =  0x20;
const FLG_V: u8 =  0x40;
const FLG_N: u8 =  0x80;
const IFLG_C: u8 =  0xFE;
const IFLG_Z: u8 =  0xFD;
const IFLG_I: u8 =  0xFB;
const IFLG_D: u8 =  0xF7;
const IFLG_B: u8 =  0xEF;
const IFLG_V: u8 =  0xBF;
const IFLG_N: u8 =  0x7F;
const IFLG_NZ: u8 =  0x7D;
const IFLG_VC: u8 =  0xBE;

macro_rules! SET_C { ($p:expr) =>{ $p |= FLG_C}}
macro_rules! SET_Z { ($p:expr) =>{ $p |= FLG_Z}}
macro_rules! SET_I { ($p:expr) =>{ $p |= FLG_I}}
macro_rules! SET_D { ($p:expr) =>{ $p |= FLG_D}}
macro_rules! SET_B { ($p:expr) =>{ $p |= FLG_B}}
macro_rules! SET_5 { ($p:expr) =>{ $p |= FLG_5}}
macro_rules! SET_V { ($p:expr) =>{ $p |= FLG_V}}
macro_rules! SET_N { ($p:expr) =>{ $p |= FLG_N}}
macro_rules! UNSET_C { ($p:expr) =>{ $p &= IFLG_C}}
macro_rules! UNSET_Z { ($p:expr) =>{ $p &= IFLG_Z}}
macro_rules! UNSET_I { ($p:expr) =>{ $p &= IFLG_I}}
macro_rules! UNSET_D { ($p:expr) =>{ $p &= IFLG_D}}
macro_rules! UNSET_B { ($p:expr) =>{ $p &= IFLG_B}}
macro_rules! UNSET_5 { ($p:expr) =>{ $p &= IFLG_5}}
macro_rules! UNSET_V { ($p:expr) =>{ $p &= IFLG_V}}
macro_rules! UNSET_N { ($p:expr) =>{ $p &= IFLG_N}}

macro_rules! UPDATE_Z { ($x: expr, $p: expr) => { if $x == 0 {SET_Z!($p)} else {UNSET_Z!($p)} } }
macro_rules! UPDATE_N { ($x: expr, $p: expr) => { if ($x&0x80) !=  0 {SET_N!($p)} else {UNSET_N!($p)} } }
macro_rules! UPDATE_NZ { ($x: expr, $p: expr) => { UPDATE_N!($x, $p); UPDATE_Z!($x, $p); } }

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
/* 80 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
/* 90 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* A0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
/* B0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* C0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* D0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* E0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
/* F0 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const clock_table: [u8;256] = [
        /* xx    00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F */
        /*  0 */  1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 0, 3, 3, 0,
        /* 10 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
        /* 20 */  3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* 30 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
        /* 40 */  1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* 50 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
        /* 60 */  1, 2, 0, 0, 0, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* 70 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
        /* 80 */  0, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0,
        /* 90 */  2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
        /* a0 */  2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* b0 */  2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
        /* c0 */  2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* d0 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
        /* e0 */  2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
        /* f0 */  2, 2, 0, 0, 0, 2, 2, 0, 1, 3, 0, 0, 0, 3, 3, 0,
];

pub struct CPU {
	a: u8,
	x: u8,
	y: u8,
	sp: u8,
	p: u8,

	pc: u16,

	clock_remain: u32,
	reset_flag: bool,

	mmu: Rc<RefCell<MMU>>,
}

impl CPU {
	pub fn new(mmu: Rc<RefCell<MMU>>) -> CPU {
		CPU {
			a: 0,
			x: 0,
			y: 0,
			sp: 0xFD,
			p:0,
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
		let mut ea: u16;
		macro_rules! IMM { 
			($ea: expr, $pc: expr) => {
				$ea = self.pc;
				$pc = self.pc +1
			}
		}
		macro_rules! ABS {
			($ea: expr, $pc: expr) => {
				$ea = mmu.read_2bytes(self.pc);
				$pc = self.pc + 2;
			}
		}
		macro_rules! ZERO_PAGE {
			($ea: expr, $pc: expr) => {
				$ea = mmu.read_1byte(self.pc) as u16;
				$pc = self.pc + 1;
			}
		}
		macro_rules! REL {
			($ea: expr, $pc: expr) => {
				let m:i8 = mmu.read_1byte($pc) as i8;
				$pc += 1;
				$ea = $pc.wrapping_add(m as u16);
			}
		}

		// Oprands
		macro_rules! BPL {
			($ea:expr) => {
				if self.p&FLG_N != 0 {
					self.pc = $ea;
				}
			}
		}
		macro_rules! JSR {
			($ea:expr) => {
				mmu.push_2bytes(0x0100 + self.sp as u16, self.pc);
				self.sp -= 2;
				self.pc = $ea;
			}
		}
		macro_rules! LDA {
			($ea:expr) => {
				self.a = mmu.read_1byte($ea);
				UPDATE_NZ!(self.a, self.p);
			}
		}
		macro_rules! LDY {
			($ea:expr) => {
				self.y = mmu.read_1byte($ea);
				UPDATE_NZ!(self.y, self.p);
			}
		}
		macro_rules! STA {
			($ea: expr) => {
				mmu.write($ea, self.a);
			}
		}
		macro_rules! RTS {
			() => {
				self.pc = mmu.pop_2bytes(0x0100 + self.sp as u16);
				self.sp -= 2;
			}
		}

		// read opcode
		let op:u8 = mmu.read_1byte(self.pc);
		self.pc += 1;

		match op {
			0x10 => { // BPL Relative
				REL!(ea, self.pc);
				BPL!(ea);
			}
			0x20 => { // JSR Absolute
				ABS!(ea, self.pc);
				JSR!(ea);
			}
			0x60 => { // RTS
				RTS!();
			}
			0x85 => { // STA ZeroPage
				ZERO_PAGE!(ea, self.pc);
				STA!(ea);
			}
			0x8D => { // STA Absolute
				ABS!(ea, self.pc);
				STA!(ea);
			}
			0xA0 => { // LDY Immediate
				IMM!(ea, self.pc);
				LDY!(ea);
			}
			0xA9 => { // LDA Immediate
				IMM!(ea, self.pc);
				LDA!(ea);
			}
			0xAD => { // LDA Absolute
				ABS!(ea, self.pc);
				LDA!(ea);
			}
			_ => {
				panic!("unsupported opcode:{:x}", op);
			}
		}
		self.clock_remain = clock_table[op as usize].into();
	}
	
	pub fn reset(&mut self) {
		let mmu = self.mmu.borrow();
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
