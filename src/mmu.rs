pub struct MMU {
	mapper: u8,
	wram: Vec<u8>,
	prom: Vec<u8>,
	crom: Vec<u8>,
}

impl MMU {
	pub fn new() -> MMU {
		MMU {
			mapper: 0,
			wram: vec![0; 0x0800],
			prom: Vec::new(),
			crom: Vec::new()
		}
	}

	pub fn read_2bytes(&mut self, addr:u16) -> u16{
		// TODO: address mapping

		let mut ret:u16 = 0;

		if addr >= 0x8000 {
			ret = self.prom[(addr - 0x8000) as usize] as u16;	
			ret |= (self.prom[(addr - 0x8000 + 1) as usize] as u16) << 8;
		}

		println!("read_2bytes({:x}) -> {:x}", addr, ret);
		return ret;
	}

	pub fn set_mapper(&mut self, m: u8) {
		self.mapper = m;
		println!("prom.mapper={}", self.mapper);
	}

	pub fn set_PROM(&mut self, prom: &[u8]) {
		self.prom = prom.to_vec();
		println!("prom.len={}", self.prom.len());
	}

	pub fn set_CROM(&mut self, crom: &[u8]) {
		self.crom = crom.to_vec();
		println!("crom.len={}", self.crom.len());
	}
}
