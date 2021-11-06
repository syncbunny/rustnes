use std::cell::RefCell;
use std::rc::Rc; 
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

const CLOCK_TABLE: [u8;256] = [
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
			p: 0x34,
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

		// PUSH/POP
		macro_rules! PUSH {
			($v: expr) => {
				mmu.write(0x0100 + (self.sp as u16), $v);
				self.sp -= 1;
			}
		}
		macro_rules! POP {
			() => {{
				self.sp += 1;
				mmu.read_1byte(0x0100 + (self.sp as u16))
			}};
		}

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
		macro_rules! ABS_INDEXED {
			($ea: expr, $pc: expr, $i: expr) => {
				$ea = mmu.read_2bytes(self.pc) + ($i as u16);
				$pc = self.pc + 2;
			}
		}
		macro_rules! ZERO_PAGE {
			($ea: expr, $pc: expr) => {
				$ea = mmu.read_1byte(self.pc) as u16;
				$pc = self.pc + 1;
			}
		}
		macro_rules! ZERO_PAGE_INDEXED {
			($ea: expr, $pc: expr, $i: expr) => {
				$ea = mmu.read_1byte(self.pc) as u16;
				$ea += $i as u16;
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
		macro_rules! INDIRECT_X {
			($ea: expr, $pc: expr) => {
				$ea = mmu.indirect_x(self.pc, self.x);
				$pc += 1;
			}
		}
		macro_rules! INDIRECT_Y {
			($ea: expr, $pc: expr) => {
				$ea = mmu.indirect_y(self.pc, self.y);
				$pc += 1;
			}
		}

		// Oprands
		macro_rules! BCC {
			($ea: expr) => {
				if self.p&FLG_C == 0 {
					self.pc = $ea;
				}
			};
		}
		macro_rules! BEQ {
			($ea: expr) => {
				if self.p&FLG_Z != 0 {
					self.pc = $ea;
				}
			};
		}
		macro_rules! BPL {
			($ea:expr) => {
				if self.p&FLG_N == 0 {
					self.pc = $ea;
				}
			}
		}
		macro_rules! BMI {
			($ea:expr) => {
				if self.p&FLG_N != 0 {
					self.pc = $ea;
				}
			}
		}
		macro_rules! BNE {
			($ea:expr) => {
				if self.p&FLG_Z == 0 {
					self.pc = $ea;
				}
			}
		}
		macro_rules! JMP {
			($ea: expr) => {
				self.pc = $ea;
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
		macro_rules! LDX {
			($ea:expr) => {
				self.x = mmu.read_1byte($ea);
				UPDATE_NZ!(self.x, self.p);
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
		macro_rules! STX {
			($ea: expr) => {
				mmu.write($ea, self.x);
			}
		}
		macro_rules! STY {
			($ea: expr) => {
				mmu.write($ea, self.y);
			}
		}
		macro_rules! INX {
			() => {
				self.x = self.x.wrapping_add(1);
				UPDATE_NZ!(self.x, self.p);
			};
		}
		macro_rules! INY {
			() => {
				self.y = self.y.wrapping_add(1);
				UPDATE_NZ!(self.y, self.p);
			};
		}
		macro_rules! DEX {
			() => {
				self.x = self.x.wrapping_sub(1);
				UPDATE_NZ!(self.x, self.p);
			}
		}
		macro_rules! DEY {
			() => {
				self.y = self.y.wrapping_sub(1);
				UPDATE_NZ!(self.y, self.p);
			}
		}
		macro_rules! RTS {
			() => {
				self.pc = mmu.pop_2bytes(0x0100 + self.sp as u16);
				self.sp += 2;
			}
		}
		macro_rules! TAX {
			() => {
				self.x = self.a;
				UPDATE_NZ!(self.x, self.p);
			}
		}
		macro_rules! TAY {
			() => {
				self.y = self.a;
				UPDATE_NZ!(self.y, self.p);
			}
		}
		macro_rules! TXS {
			() => {
				self.sp = self.x;
				// NO flags update
			}
		}
		macro_rules! SEC {
			() => {
				SET_C!(self.p);
			}
		}
		macro_rules! SEI {
			() => {
				SET_I!(self.p);
			}
		}
		macro_rules! CLC {
			() => {
				UNSET_C!(self.p);
			};
		}
		macro_rules! CLD {
			() => {
				UNSET_D!(self.p);
			};
		}
		macro_rules! CLI {
			() => {
				UNSET_I!(self.p);
			};
		}
		macro_rules! AND {
			($ea: expr) => {
				self. a &= mmu.read_1byte(ea);
				UPDATE_NZ!(self.a, self.p);
			};
		}
		macro_rules! ORA {
			($ea: expr) => {
				self.a |= mmu.read_1byte($ea);
				UPDATE_NZ!(self.a, self.p);
			}
		}
		macro_rules! ASL_A {
			() => {
				if self.a & 0x80 == 0 {
					UNSET_C!(self.p);
				} else {
					SET_C!(self.p);
				}
				self.a <<= 1;
				UPDATE_NZ!(self.a, self.p);
			}
		}
		macro_rules! ROL_A {
			() => {
				let a = self.a;
				self.a <<= 1;
				self.a |= if (self.p & FLG_C) != 0 {
					0x01
				} else {
					0x00
				};
				if (a & 0x80) != 0 {
					SET_C!(self.p);
				} else {
					UNSET_C!(self.p);
				}
				UPDATE_NZ!(self.a, self.p);
			}
		}
		macro_rules! INC {
			($ea: expr) => {
				let m:u8 = mmu.read_1byte($ea);
				mmu.write($ea, m+1);
				UPDATE_NZ!(m+1, self.p);
			}
		}
		macro_rules! ADC {
			($ea: expr) => {
				let m:u8 = mmu.read_1byte($ea);
				let c:u8 = if self.p & FLG_C != 0 {1} else {0};
				let t:u16 = self.a as u16 + m as u16 + c as u16;

				if t > 0x00FFu16 {
					SET_C!(self.p);
				} else {
					UNSET_C!(self.p);
				}
				let new_a:u8 = (t & 0x00FFu16) as u8;
                if ((self.a ^ new_a) & (m ^ new_a) & 0x80) == 0x80 {
					SET_V!(self.p);
				} else {
					UNSET_V!(self.p);
				}

				self.a = new_a;
				UPDATE_NZ!(self.a, self.p);
			};
		}
		macro_rules! CMP {
			($ea: expr) => {
				let m:u8 = mmu.read_1byte(ea);
				if self.a >= m {
					SET_C!(self.p);
				} else {
					UNSET_C!(self.p);
				}
				UPDATE_NZ!(self.a.wrapping_sub(m), self.p);
			};
		}
		macro_rules! CPX {
			($ea: expr) => {
				let m:u8 = mmu.read_1byte(ea);
				if self.x >= m {
					SET_C!(self.p);
				} else {
					UNSET_C!(self.p);
				}
				UPDATE_NZ!(self.x.wrapping_sub(m), self.p);
			};
		}
		macro_rules! PHA {
			() => {
				PUSH!(self.a);	
			}
		}
		macro_rules! PLA {
			() => {
				self.a = POP!();
				UPDATE_NZ!(self.a, self.p);
			}
		}
		// read opcode
		let op:u8 = mmu.read_1byte(self.pc);
		self.pc += 1;

		match op {
			0x09 => { // ORA Immediate
				IMM!(ea, self.pc);
				ORA!(ea);
			}
			0x0A => { // ASL Accumurator
				ASL_A!();
			}
			0x10 => { // BPL Relative
				REL!(ea, self.pc);
				BPL!(ea);
			}
			0x18 => { // CLC
				CLC!();
			}
			0x20 => { // JSR Absolute
				ABS!(ea, self.pc);
				JSR!(ea);
			}
			0x29 => { // AND Immediate
				IMM!(ea, self.pc);
				AND!(ea);
			}
			0x2A => { // ROL Accumurator
				ROL_A!();
			}
			0x30 => { // BMI Relative
				REL!(ea, self.pc);
				BMI!(ea);
			}
			0x38 => { // SEC
				SEC!();
			}
			0x48 => { // PHA
				PHA!();
			}
			0x4C => { // JMP Absolute
				ABS!(ea, self.pc);
				JMP!(ea);
			}
			0x58 => { // CLI
				CLI!();
			}
			0x60 => { // RTS
				RTS!();
			}
			0x68 => { // PLA
				PLA!();
			}
			0x69 => { // ADC Immediate
				IMM!(ea, self.pc);
				ADC!(ea);
			}
			0x78 => { // SEI
				SEI!();
			}
			0x85 => { // STA ZeroPage
				ZERO_PAGE!(ea, self.pc);
				STA!(ea);
			}
			0x88 => { // DEY
				DEY!();
			}
			0x8C => { // STY Absolute
				ABS!(ea, self.pc);
				STY!(ea);
			}
			0x8E => { // STX Absolute
				ABS!(ea, self.pc);
				STX!(ea);
			}
			0x8D => { // STA Absolute
				ABS!(ea, self.pc);
				STA!(ea);
			}
			0x90 => { // BCC Relative
				REL!(ea, self.pc);
				BCC!(ea);
			}
			0x91 => { // STA Indirect Y
				INDIRECT_Y!(ea, self.pc);
				STA!(ea);
			}
			0x95 => { // STA ZeroPage, X
				ZERO_PAGE_INDEXED!(ea, self.pc, self.x);
				STA!(ea);
			}
			0x99 => { // STA Absolute, Y
				ABS_INDEXED!(ea, self.pc, self.y);
				STA!(ea);
			}
			0x9A => { // TXS
				TXS!();
			}
			0xA0 => { // LDY Immediate
				IMM!(ea, self.pc);
				LDY!(ea);
			}
			0xA1 => { // LDA Indirect, X
				INDIRECT_X!(ea, self.pc);
				LDA!(ea);
			}
			0xA2 => { // LDX Immediate
				IMM!(ea, self.pc);
				LDX!(ea);
			}
			0xA5 => { // LDA ZeroPage
				ZERO_PAGE!(ea, self.pc);
				LDA!(ea);
			}
			0xA8 => { // TAY
				TAY!();
			}
			0xA9 => { // LDA Immediate
				IMM!(ea, self.pc);
				LDA!(ea);
			}
			0xAA => { // TAX
				TAX!();
			}
			0xAD => { // LDA Absolute
				ABS!(ea, self.pc);
				LDA!(ea);
			}
			0xB1 => { // LDA Indirect, Y
				INDIRECT_Y!(ea, self.pc);
				LDA!(ea);
			}
			0xB5 => { // LDA ZeroPage, X
				ZERO_PAGE_INDEXED!(ea, self.pc, self.x);
				LDA!(ea);
			}
			0xB9 => { // LDA Abusolute, Y
				ABS_INDEXED!(ea, self.pc, self.y);
				LDA!(ea);
			}
			0xBD => { // LDA Absolute, X
				ABS_INDEXED!(ea, self.pc, self.x);
				LDA!(ea);
			}
			0xC5 => { // CMP ZeroPage
				ZERO_PAGE!(ea, self.pc);
				CMP!(ea);
			}
			0xC8 => { // INY
				INY!();
			}
			0xC9 => { // CMP Immediate
				IMM!(ea, self.pc);
				CMP!(ea);
			}
			0xCA => { // DEX
				DEX!();
			}
			0xD0 => { // BNE Relative
				REL!(ea, self.pc);
				BNE!(ea);
			}
			0xD8 => { // CLD
				CLD!();
			}
			0xE0 => { // CPX Immediate
				IMM!(ea, self.pc);
				CPX!(ea);
			}
			0xE6 => { // INC ZeroPage
				ZERO_PAGE!(ea, self.pc);
				INC!(ea);
			}
			0xE8 => { // INX
				INX!();
			}
			0xF0 => { // BEQ Relative
				REL!(ea, self.pc);
				BEQ!(ea);
			}
			_ => {
				panic!("unsupported opcode:{:x}", op);
			}
		}
		self.clock_remain = CLOCK_TABLE[op as usize].into();
		SET_5!(self.p); // bit 5 is always 1

		self.dump();
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

	fn dump(&self) {
		println!("CPU:PC={:04X}, A={:02X}, X={:02X}, Y={:02X}, S={:02X}, P={:02X}", self.pc, self.a, self.x, self.y, self.sp, self.p);
	}
}
