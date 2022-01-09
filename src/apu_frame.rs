use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::apu_square::*;
use crate::apu_triangle::*;
use crate::apu_noise::*;
use crate::events::*;

const SEQ_MODE_MASK: u8 = 0x80;
const NO_IRQ_MASK:u8 = 0x40;

pub struct APUFrame {
	pub interrupted: bool,

	cr: u8,
	seq: u8,
	square1: Rc<RefCell<APUSquare>>,
	square2: Rc<RefCell<APUSquare>>,
	triangle: Rc<RefCell<APUTriangle>>,
	noise: Rc<RefCell<APUNoise>>,
	event_queue: Arc<Mutex<EventQueue>>
}

impl APUFrame {
	pub fn new(
			square1: Rc<RefCell<APUSquare>>,
			square2: Rc<RefCell<APUSquare>>,
			triangle: Rc<RefCell<APUTriangle>>,
			noise: Rc<RefCell<APUNoise>>,
			event_queue: Arc<Mutex<EventQueue>>
		) -> APUFrame {
		APUFrame {
			interrupted: false,

			cr: 0,
			seq: 0,
			square1: square1,
			square2: square2,
			triangle: triangle,
			noise: noise,
			event_queue: event_queue,
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

					self.square1.borrow_mut().sweep_clock();
					self.square2.borrow_mut().sweep_clock();

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

					self.square1.borrow_mut().sweep_clock();
					self.square2.borrow_mut().sweep_clock();

					self.triangle.borrow_mut().linear_clock();

					self.square1.borrow_mut().envelope_clock();
					self.square2.borrow_mut().envelope_clock();
					self.noise.borrow_mut().envelope_clock();

					if self.cr & NO_IRQ_MASK == 0 {
						let mut queue = self.event_queue.lock().unwrap();
						queue.push(Event::new(EventType::IRQ));
						self.interrupted = true;
					}
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

					self.square1.borrow_mut().sweep_clock();
					self.square2.borrow_mut().sweep_clock();

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

					self.square1.borrow_mut().sweep_clock();
					self.square2.borrow_mut().sweep_clock();

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
