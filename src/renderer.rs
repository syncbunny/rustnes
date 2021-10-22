extern crate sdl2;
extern crate gl;

use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::os::raw::c_void;
use sdl2::*;
use sdl2::event::Event;
use sdl2::video::GLProfile;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use gl::types::{GLuint};
use crate::io::*;

pub struct Renderer {
	io: Arc<Mutex<IO>>,
	tex_id: u32,
	tex_data: Vec<u8>,
	sdl_context: Sdl,
	window: Window,
}

impl Renderer {
	pub fn new(io:Arc<Mutex<IO>>) -> Renderer {
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

		let mut ret = Renderer {
			io: io,
			tex_id: 0,
			tex_data: vec![0; 256*240*3],
			sdl_context: sdl_context,
			window: window
		};

		ret.init_gl();
		return ret;
	}

	pub fn event_loop (&mut self) {
		let mut event_pump = self.sdl_context.event_pump().unwrap();

		'running: loop {
			unsafe {
				gl::ClearColor(0.6, 0.0, 0.8, 1.0);
				//gl::Clear(gl::COLOR_BUFFER_BIT);
			}

        		self.window.gl_swap_window();
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

	fn init_gl(&mut self) {
		unsafe {
			gl::GenTextures(1, &mut self.tex_id);
			gl::BindTexture(gl::TEXTURE_2D, self.tex_id);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 256,  240, 0, gl::RGB, gl::UNSIGNED_BYTE, self.tex_data.as_ptr() as *const u8 as *const c_void);
		}
	}
}
