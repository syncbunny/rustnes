use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::io::*;

pub struct Renderer {
	io: Arc<Mutex<IO>>
}

impl Renderer {
	pub fn new(io:Arc<Mutex<IO>>) -> Renderer {
		Renderer {
			io: io
		}
	}

	pub fn event_loop (& self) {
	}
}
