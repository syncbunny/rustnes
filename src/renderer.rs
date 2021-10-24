extern crate sdl2;
extern crate gl;

use std::mem;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;
use std::rc::Rc;
use std::ptr;
use std::string::String;
use std::ffi::CString;
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
	vao: u32,
	tex_data: Vec<u8>,
	sdl_context: Sdl,
	shader_program: u32,
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
			vao: 0,
			tex_data: vec![0; 256*240*3],
			shader_program: 0,
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
			// create texture
			gl::GenTextures(1, &mut self.tex_id);
			gl::BindTexture(gl::TEXTURE_2D, self.tex_id);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 256,  240, 0, gl::RGB, gl::UNSIGNED_BYTE, self.tex_data.as_ptr() as *const u8 as *const c_void);

			// create object
			let position_data:[f32;12] = [
				-1.0,  1.0, 0.0,
				-1.0, -1.0, 0.0,
				 1.0,  1.0, 0.0,
				 1.0, -1.0, 0.0,
			];
			gl::GenVertexArrays(1, &mut self.vao);
			gl::BindVertexArray(self.vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vao);
			gl::BufferData(gl::ARRAY_BUFFER, ((mem::size_of::<f32> as u32)*12) as isize, &position_data[0] as *const f32 as *const c_void, gl::STATIC_DRAW);

			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);

			gl::BindVertexArray(0);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		self.create_program();
	}

	fn create_program(&mut self) {
		let vs_src:CString = CString::new("
			#version 150 core
			in vec4 position;
			out vec2 texcoord;

			void main() {
				gl_Position = position;
				texcoord = vec2(gl_VertexID/2, gl_VertexID%2);
			}
		").unwrap();
		let fs_src:CString = CString::new("
			#version 150 core
			
			uniform sampler2D image;
			in vec2 texcoord;
			out vec4 fragment;

			void main() {
				fragment = texture(image, texcoord);
			}
		").unwrap();
		unsafe {
			self.shader_program = gl::CreateProgram();
		}

		unsafe {
			println!("compile vertex shader");
			let vobj = gl::CreateShader(gl::VERTEX_SHADER);
			gl::ShaderSource(vobj, 1, &mut vs_src.as_ptr(), ptr::null());
			gl::CompileShader(vobj);
			self.print_shader_log(vobj, "VertexShader");
			println!("done.");
			gl::AttachShader(self.shader_program, vobj);
			gl::DeleteShader(vobj);
		}

		unsafe {
			println!("compile fragment shader");
			let vobj = gl::CreateShader(gl::FRAGMENT_SHADER);
			gl::ShaderSource(vobj, 1, &mut fs_src.as_ptr(), ptr::null());
			gl::CompileShader(vobj);
			self.print_shader_log(vobj, "FragmentShader");
			println!("done.");
			gl::AttachShader(self.shader_program, vobj);
			gl::DeleteShader(vobj);
		}
		
	}

	fn print_shader_log(&self, shader: u32, msg: &str) -> i32 {
		let mut buf_size: i32 = 0;

		unsafe {
			gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut buf_size);
		}
		if buf_size > 1 {
			let mut length = 0_i32;
			let mut log:Vec<u8> = Vec::with_capacity(buf_size as usize);
			unsafe {
				gl::GetShaderInfoLog(shader, buf_size, &mut length, log.as_mut_ptr() as *mut i8);
			}
			let log_str = std::str::from_utf8(&log).unwrap();
			println!("{}:{}", msg, log_str);
		}

		return buf_size;
	}
}
