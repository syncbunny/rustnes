pub struct APU {
	sw1c1: u8,         // 0x4000
	sw1c2: u8,         // 0x4001
	sw1fq1: u8,        // 0x4002
	sw1fq2: u8,        // 0x4003
	sw2c1: u8,         // 0x4004
	sw2c2: u8,         // 0x4005
	sw2fq1: u8,        // 0x4006
	sw2fq2: u8,        // 0x4007
	twc: u8,           // 0x4008
	twfq1: u8,         // 0x400A
	twfq2: u8,         // 0x400B
	nc: u8,            // 0x400C
	nfq1: u8,          // 0x400E
	nfq2: u8,          // 0x400F
	dmc1: u8,          // 0x4010
	dmc2: u8,          // 0x4011
	dmc3: u8,          // 0x4012
	dmc4: u8,          // 0x4013
	ch_ctrl: u8,       // 0x4015
	frame_counter: u8, // 0x4017
}

impl APU {
	pub fn new() -> APU {
		APU {
			sw1c1: 0,
			sw1c2: 0,
			sw1fq1: 0,
			sw1fq2: 0,
			sw2c1: 0,
			sw2c2: 0,
			sw2fq1: 0,
			sw2fq2: 0,
			twc: 0,
			twfq1: 0,
			twfq2: 0,
			nc: 0,
			nfq1: 0,
			nfq2: 0,
			dmc1: 0,
			dmc2: 0,
			dmc3: 0,
			dmc4: 0,
			ch_ctrl: 0,
			frame_counter: 0,
		}
	}

	pub fn reset(&mut self) {
		// TODO
	}

	pub fn set_dmc1(&mut self, v: u8) {
		// TODO
	}

	pub fn set_ch_ctrl(&mut self, v: u8) {
		// TODO
	}

	pub fn set_frame_counter(&mut self, v: u8) {
		// TODO
	}
}
