extern crate sdl2;
extern crate gl;

use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use sdl2::*;
use sdl2::event::Event;
use sdl2::video::GLProfile;
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
