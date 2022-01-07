pub struct APUEnvelope {
	pub val: u8,
	cr: u8,

	reset: bool,
	clock: u8,
}

const LOOP_MASK:u8 = 0x20;
const OFF_MASK:u8 = 0x10;
const CLOCK_DIV_MASK:u8 = 0x0F;

impl APUEnvelope {
	pub fn new() -> APUEnvelope {
		APUEnvelope{
			cr: 0,

			val: 0,
			reset: false,
			clock: 0,
		}
	}

	pub fn set_cr(&mut self, v: u8) {
		self.cr = v;
	}

	pub fn clock(&mut self) {
		if self.reset {
			self.val = 0x0F;
			self.clock = self.cr & CLOCK_DIV_MASK;
			self.reset = false;
		} else {
			if self.clock == 0 {
				if self.val == 0 {
					if self.cr & LOOP_MASK != 0 {
						self.val = 0x0F;
					}
				} else {
					self.val -= 1;
				}
			} else {
				self.clock -= 1;
			}
		}
	}

	pub fn reset(&mut self) {
		self.reset = true;
	}

	pub fn val(&self) -> u8 {
		if self.cr & OFF_MASK == 0 {
			return self.val;
		} else {
			return self.cr & CLOCK_DIV_MASK;
		}
	}
}
