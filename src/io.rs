pub struct IO {
	vram: Vec<u8>
}


impl IO {
	pub fn new() -> IO {
		IO {
			vram: vec![0; 245760*2] // 256*240*4 * 2
		}
	}
}
