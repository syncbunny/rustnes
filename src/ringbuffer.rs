pub struct RingBuffer<T:Copy> {
	wp: usize,
	rp: usize,
	remain: usize,
	cap: usize,
	data: Vec<T>
}

impl<T:Copy> RingBuffer<T> {
	pub fn new(cap: usize, v:T) -> RingBuffer<T> {
		RingBuffer {
			wp: 0,
			rp: 0,
			remain: 0,
			cap: cap,
			data: vec![v; cap]
		}
	}

	pub fn write(&mut self, v:T) -> bool {
		if self.remain < self.cap {
			self.data[self.wp] = v;
			self.wp += 1;
			if self.wp >= self.cap {
				self.wp = 0;
			}
			self.remain += 1;
			return true;
		} else {
			return false;
		}
	}

	pub fn read(&mut self) -> Option<T> {
		if self.remain > 0 {
			let t = self.data[self.rp];
			self.rp += 1;
			if self.rp >= self.cap {
				self.rp = 0;
			}
			self.remain -= 1;
			return Some(t);
		} else {
			return None;
		}
	}
}
