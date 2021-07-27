pub struct PPU {
	cr1: u8,  // Control Register 1
	cr2: u8,  // Control Register 1
	sr: u8,	  // Status Register
}

impl PPU {
	pub fn new() -> PPU {
		PPU {
			cr1: 0,
			cr2: 0,
			sr: 0
		}

	}

	pub fn reset(&self) {
	}

	pub fn set_cr1(&mut self, n:u8) {
		self.cr1 = n;
	}

	pub fn set_cr2(&mut self, n:u8) {
		self.cr2 = n;
	}

	pub fn get_sr(&self) -> u8 {
		return self.sr;
	}
}
