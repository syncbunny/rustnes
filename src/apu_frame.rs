use std::cell::RefCell;
use std::rc::Rc;
use crate::apu_triangle::*;

const SEQ_MODE_MASK: u8 = 0x80;

pub struct APUFrame {
	cr: u8,
	seq: u8,
	triangle: Rc<RefCell<APUTriangle>>
}

impl APUFrame {
	pub fn new(triangle: Rc<RefCell<APUTriangle>>) -> APUFrame {
		APUFrame {
			cr: 0,
			seq: 0,
			triangle: triangle
		}
	}

	pub fn clock(&mut self) {
		if self.cr & SEQ_MODE_MASK == 0 {
			// 4-step
			match self.seq {
				0 => {
					self.triangle.borrow_mut().linear_clock();
				}
				1 => {
					self.triangle.borrow_mut().length_clock();
					self.triangle.borrow_mut().linear_clock();
				}
				2 => {
					self.triangle.borrow_mut().linear_clock();
				}
				3 => {
					self.triangle.borrow_mut().length_clock();
					self.triangle.borrow_mut().linear_clock();
				}
				_ => {}
			}

			self.seq += 1;
			if self.seq >= 4 {
				self.seq = 0;
			}
		} else {
			// 5-step
			match self.seq {
				0 => {
					self.triangle.borrow_mut().length_clock();
					self.triangle.borrow_mut().linear_clock();
				}
				1 => {
					self.triangle.borrow_mut().linear_clock();
				}
				2 => {
					self.triangle.borrow_mut().length_clock();
					self.triangle.borrow_mut().linear_clock();
				}
				3 => {
					self.triangle.borrow_mut().linear_clock();
				}
				4 => {
					/* NOP */
				}
				_ => {}
			}

			self.seq += 1;
			if self.seq >= 5 {
				self.seq = 0;
			}
		}
	}

	pub fn set_cr(&mut self, v: u8) -> u8 {
		self.cr = v;
		return self.cr;
	}
}
