pub struct APUSquare{
	cr1: u8,
	cr2: u8,
	fq1: u8,
	fq2: u8,

	length_counter: u8,
}

impl APUSquare {
	pub fn new() -> APUSquare {
		APUSquare {
			cr1: 0,
			cr2: 0,
			fq1: 0,
			fq2: 0,

			length_counter: 0,
		}
	}

	pub fn clock(&mut self) {
	}

	pub fn set_cr1(&mut self, v:u8) -> u8 {
		self.cr1 = v;
		return self.cr1;
	}

	pub fn set_cr2(&mut self, v:u8) -> u8 {
		self.cr2 = v;
		return self.cr2;
	}

	pub fn set_fq1(&mut self, v:u8) -> u8 {
		self.cr1 = v;
		return self.fq1;
	}

	pub fn set_fq2(&mut self, v:u8) -> u8 {
		self.cr2 = v;
		return self.fq2;
	}

	pub fn set_ch_ctrl(&mut self, v:u8) {
		if v == 0 {
			self.length_counter = 0;
		}
	}
}
