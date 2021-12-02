pub struct Pad {
	a: [u8;2],
	b: [u8;2],
	select: [u8;2],
	start: [u8;2],
	up: [u8;2],
	down: [u8;2],
	left: [u8;2],
	right: [u8;2],
	last_out: u8,
	in_count_1: u8,
	in_count_2: u8,

	a_: [u8;2],
	b_: [u8;2],
	select_: [u8;2],
	start_: [u8;2],
	up_: [u8;2],
	down_: [u8;2],
	left_: [u8;2],
	right_: [u8;2],
}

impl Pad {
	pub fn new() -> Pad {
		Pad {
			a: [0, 0],
			b: [0, 0],
			select: [0, 0],
			start: [0, 0],
			up: [0, 0],
			down: [0, 0],
			left: [0, 0],
			right: [0, 0],
			last_out: 0,
			in_count_1: 0,
			in_count_2: 0,

			a_: [0, 0],
			b_: [0, 0],
			select_: [0, 0],
			start_: [0, 0],
			up_: [0, 0],
			down_: [0, 0],
			left_: [0, 0],
			right_: [0, 0],
		}
	}

	pub fn out(&mut self, n:u8) {
		if (self.last_out == 0x01 && (n & 0x01) == 0x00) {
			self.strobe();
			self.reset_count();
		}
		self.last_out = n & 0x01;
	}

	pub fn in1(&mut self) -> u8 {
		let ret:u8;

		match self.in_count_1%8 {
			0 => { ret = self.a[0]; }
			1 => { ret = self.b[0]; }
			2 => { ret = self.select[0]; }
			3 => { ret = self.start[0]; }
			4 => { ret = self.up[0]; }
			5 => { ret = self.down[0]; }
			6 => { ret = self.left[0]; }
			7 => { ret = self.right[0]; }
			_ => { panic!("unexcepted!")}
		}
		
		self.in_count_1 += 1;
		self.in_count_1 %= 8;

		return ret;
	}

	pub fn in2(&mut self) -> u8 {
		let ret:u8;

		match self.in_count_2%8 {
			0 => { ret = self.a[1]; }
			1 => { ret = self.b[1]; }
			2 => { ret = self.select[1]; }
			3 => { ret = self.start[1]; }
			4 => { ret = self.up[1]; }
			5 => { ret = self.down[1]; }
			6 => { ret = self.left[1]; }
			7 => { ret = self.right[1]; }
			_ => { panic!("unexcepted!")}
		}
		
		self.in_count_2 += 1;
		self.in_count_2 %= 8;

		return ret;
	}

	pub fn set_a(&mut self, n:u32, v:u8) {
		self.a_[n as usize] = v;
	}

	pub fn set_b(&mut self, n:u32, v:u8) {
		self.b_[n as usize] = v;
	}

	pub fn set_start(&mut self, n:u32, v:u8) {
		self.start_[n as usize] = v;
	}

	pub fn set_select(&mut self, n:u32, v:u8) {
		self.select_[n as usize] = v;
	}

	pub fn set_up(&mut self, n:u32, v:u8) {
		self.up_[n as usize] = v;
	}

	pub fn set_down(&mut self, n:u32, v:u8) {
		self.down_[n as usize] = v;
	}

	pub fn set_left(&mut self, n:u32, v:u8) {
		self.left_[n as usize] = v;
	}

	pub fn set_right(&mut self, n:u32, v:u8) {
		self.right_[n as usize] = v;
	}

	fn strobe(&mut self) {
		self.a[0] = self.a_[0];
		self.a[1] = self.a_[1];
		self.b[0] = self.b_[0];
		self.b[1] = self.b_[1];
		self.select[0] = self.select_[0];
		self.select[1] = self.select_[1];
		self.start[0] = self.start_[0];
		self.start[1] = self.start_[1];
		self.up[0] = self.up_[0];
		self.up[1] = self.up_[1];
		self.down[0] = self.down_[0];
		self.down[1] = self.down_[1];
		self.left[0] = self.left_[0];
		self.left[1] = self.left_[1];
		self.right[0] = self.right_[0];
		self.right[1] = self.right_[1];
	}

	fn reset_count(&mut self) {
		self.in_count_1 = 0;
		self.in_count_2 = 0;
	}
}

