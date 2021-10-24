pub struct IO {
	pub vram: Vec<u8>
}

impl IO {
	pub fn new() -> IO {
		IO {
			vram: vec![0; 256*240*3]
		}
	}

	pub fn draw_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
		self.vram[((y*256 +x)*3 +0) as usize] = r;
		self.vram[((y*256 +x)*3 +1)as usize] = g;
		self.vram[((y*256 +x)*3 +2)as usize] = 155;
	}
}