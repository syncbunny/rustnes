use std::cell::RefCell;
use std::rc::Rc;
use crate::apu_square::*;
use crate::apu_triangle::*;
use crate::apu_noise::*;

const SEQ_MODE_MASK: u8 = 0x80;

pub struct APUFrame {
	cr: u8,
	seq: u8,
	square1: Rc<RefCell<APUSquare>>,
	square2: Rc<RefCell<APUSquare>>,
	triangle: Rc<RefCell<APUTriangle>>,
	noise: Rc<RefCell<APUNoise>>
}

impl APUFrame {
	pub fn new(
			square1: Rc<RefCell<APUSquare>>,
			square2: Rc<RefCell<APUSquare>>,
			triangle: Rc<RefCell<APUTriangle>>,
			noise: Rc<RefCell<APUNoise>>
		) -> APUFrame {
		APUFrame {
			cr: 0,
			seq: 0,
			square1: square1,
			square2: square2,
			triangle: triangle,
			noise: noise
		}
	}

	pub fn clock(&mut self) {
		if self.cr & SEQ_MODE_MASK == 0 {
			// 4-step
			match self.seq {
				0 => {
					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				1 => {
					self.square1.borrow_mut().length_clock();
					self.square2.borrow_mut().length_clock();
					self.triangle.borrow_mut().length_clock();
					self.noise.borrow_mut().length_clock();

					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				2 => {
					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				3 => {
					self.square1.borrow_mut().length_clock();
					self.square2.borrow_mut().length_clock();
					self.triangle.borrow_mut().length_clock();
					self.noise.borrow_mut().length_clock();

					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
					// TODO IRQ
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
					self.square1.borrow_mut().length_clock();
					self.square2.borrow_mut().length_clock();
					self.triangle.borrow_mut().length_clock();
					self.noise.borrow_mut().length_clock();

					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				1 => {
					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				2 => {
					self.square1.borrow_mut().length_clock();
					self.square2.borrow_mut().length_clock();
					self.triangle.borrow_mut().length_clock();
					self.noise.borrow_mut().length_clock();

					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
				}
				3 => {
					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();
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
