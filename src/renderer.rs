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
use sdl2::keyboard::Keycode;
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
		let sdl_context = sdl2::init().unwrap();
		let video_subsystem = sdl_context.video().unwrap();
    
		let gl_attr = video_subsystem.gl_attr();
		gl_attr.set_context_profile(GLProfile::Core);
		gl_attr.set_context_version(3, 3);

		let window = video_subsystem.window("Window", 256, 240)
			.opengl()
			.build()
			.unwrap();

		let ctx = window.gl_create_context().unwrap();
		gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    
		let mut event_pump = sdl_context.event_pump().unwrap();

		'running: loop {
			unsafe {
				gl::ClearColor(0.6, 0.0, 0.8, 1.0);
				//gl::Clear(gl::COLOR_BUFFER_BIT);
			}

        		window.gl_swap_window();
			for event in event_pump.poll_iter() {
				match event {
					Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
						break 'running
					},
					_ => {}
				}
        		}
        		::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    		}
	}
}
