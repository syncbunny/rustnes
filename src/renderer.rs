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
use sdl2::video::GLContext;
use gl::types::{GLuint};
use crate::io::*;

pub struct Renderer {
	io: Arc<Mutex<IO>>,
	tex_id: u32,
	vao: u32,
	vbo: u32,
	tex_data: Vec<u8>,
	sdl_context: Sdl,
	shader_program: u32,
	window: Window,
	gl_context: GLContext,
}

impl Renderer {
	pub fn new(io:Arc<Mutex<IO>>) -> Renderer {
		let sdl_context = sdl2::init().unwrap();
		let video_subsystem = sdl_context.video().unwrap();
		let controller_subsystem = sdl_context.game_controller().unwrap();

		let nr_controller = controller_subsystem.num_joysticks().unwrap();
		//println!("Number of controllers: {}", nr_controller);

		let gl_attr = video_subsystem.gl_attr();
		gl_attr.set_context_profile(GLProfile::Core);
		gl_attr.set_context_version(3, 3);

		let window = video_subsystem.window("Window", 256, 240)
			.opengl()
			.build()
			.unwrap();

		let ctx = window.gl_create_context().unwrap();
		gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

		debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
		debug_assert_eq!(gl_attr.context_version(), (3, 3));

		let mut ret = Renderer {
			io: io,
			tex_id: 0,
			vao: 0,
			vbo: 0,
			tex_data: vec![0; 256*240*3],
			shader_program: 0,
			sdl_context: sdl_context,
			gl_context: ctx,
			window: window
		};

		ret.init_gl();
		return ret;
	}

	pub fn event_loop (&mut self) {
		let mut event_pump = self.sdl_context.event_pump().unwrap();

		'running: loop {
			self.check_gl_error(line!());
			unsafe {
				gl::ClearColor(0.6, 0.0, 0.8, 1.0);
				self.check_gl_error(line!());
				gl::Clear(gl::COLOR_BUFFER_BIT);
				self.check_gl_error(line!());
			}

        		//self.window.gl_swap_window();
			self.check_gl_error(line!());
			{
				let mut io = self.io.lock().unwrap();
				self.tex_data[0..].copy_from_slice(&io.vram[0..]);
			}

			unsafe{
				gl::Clear(gl::COLOR_BUFFER_BIT);

				gl::BindTexture(gl::TEXTURE_2D, self.tex_id);
				self.check_gl_error(line!());
				gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 256, 240, 0, gl::RGB, gl::UNSIGNED_BYTE, self.tex_data.as_ptr() as *const c_void);
				self.check_gl_error(line!());

				gl::UseProgram(self.shader_program);
				self.check_gl_error(line!());
				gl::BindVertexArray(self.vao);
				self.check_gl_error(line!());
				gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
				//gl::DrawArrays(gl::LINES, 0, 4);
				self.check_gl_error(line!());
				gl::BindVertexArray(0);
				self.check_gl_error(line!());
			}
        		self.window.gl_swap_window();

			for event in event_pump.poll_iter() {
				match event {
					Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
						break 'running
					},

					Event::KeyDown { keycode: Some(Keycode::X), .. } => {
						self.io.lock().unwrap().pad.set_a(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::X), .. } => {
						self.io.lock().unwrap().pad.set_a(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
						self.io.lock().unwrap().pad.set_b(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Z), .. } => {
						self.io.lock().unwrap().pad.set_b(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
						self.io.lock().unwrap().pad.set_start(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Return), .. } => {
						self.io.lock().unwrap().pad.set_start(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::RShift), .. } => {
						self.io.lock().unwrap().pad.set_select(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::RShift), .. } => {
						self.io.lock().unwrap().pad.set_select(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
						self.io.lock().unwrap().pad.set_up(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
						self.io.lock().unwrap().pad.set_up(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
						self.io.lock().unwrap().pad.set_up(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
						self.io.lock().unwrap().pad.set_up(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
						self.io.lock().unwrap().pad.set_down(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
						self.io.lock().unwrap().pad.set_down(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
						self.io.lock().unwrap().pad.set_left(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Left), .. } => {
						self.io.lock().unwrap().pad.set_left(0, 0);
					}

					Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
						self.io.lock().unwrap().pad.set_right(0, 1);
					}
					Event::KeyUp { keycode: Some(Keycode::Right), .. } => {
						self.io.lock().unwrap().pad.set_right(0, 0);
					}

					_ => {}
				}
        		}
			//println!("render_loop:");
        		::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    		}
	}

	fn init_gl(&mut self) {
		unsafe {
			// create texture
			gl::GenTextures(1, &mut self.tex_id);
			self.check_gl_error(line!());
			gl::BindTexture(gl::TEXTURE_2D, self.tex_id);
			self.check_gl_error(line!());

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			self.check_gl_error(line!());
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			self.check_gl_error(line!());
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			self.check_gl_error(line!());

			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 256,  240, 0, gl::RGB, gl::UNSIGNED_BYTE, self.tex_data.as_ptr() as *const c_void);
			self.check_gl_error(line!());

			// create object
			let position_data:Vec<f32> = vec![
				-1.0,  1.0, 0.0,
				-1.0, -1.0, 0.0,
				 1.0,  1.0, 0.0,
				 1.0, -1.0, 0.0,
			];
			gl::GenVertexArrays(1, &mut self.vao);
			self.check_gl_error(line!());
			gl::BindVertexArray(self.vao);
			self.check_gl_error(line!());

			gl::GenBuffers(1, &mut self.vbo);
			self.check_gl_error(line!());
			gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
			self.check_gl_error(line!());
			gl::BufferData(gl::ARRAY_BUFFER, (mem::size_of::<f32>()*12) as isize, position_data.as_ptr() as *const c_void, gl::STATIC_DRAW);
			self.check_gl_error(line!());

			gl::EnableVertexAttribArray(0);
			self.check_gl_error(line!());
			gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const c_void);
			self.check_gl_error(line!());

			gl::BindVertexArray(0);
			self.check_gl_error(line!());
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			self.check_gl_error(line!());
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
			self.check_gl_error(line!());
			gl::CompileShader(vobj);
			self.check_gl_error(line!());
			self.print_shader_log(vobj, "VertexShader");
			println!("done.");
			gl::AttachShader(self.shader_program, vobj);
			self.check_gl_error(line!());
			gl::DeleteShader(vobj);
			self.check_gl_error(line!());
		}

		unsafe {
			println!("compile fragment shader");
			let vobj = gl::CreateShader(gl::FRAGMENT_SHADER);
			gl::ShaderSource(vobj, 1, &mut fs_src.as_ptr(), ptr::null());
			self.check_gl_error(line!());
			gl::CompileShader(vobj);
			self.check_gl_error(line!());
			self.print_shader_log(vobj, "FragmentShader");
			println!("done.");
			gl::AttachShader(self.shader_program, vobj);
			self.check_gl_error(line!());
			gl::DeleteShader(vobj);
			self.check_gl_error(line!());
		}
	
		unsafe {
			gl::BindAttribLocation(self.shader_program, 0, CString::new("position").unwrap().as_ptr());
			self.check_gl_error(line!());
			gl::BindFragDataLocation(self.shader_program, 0, CString::new("fragment").unwrap().as_ptr());
			self.check_gl_error(line!());
			gl::LinkProgram(self.shader_program);
			self.check_gl_error(line!());
			self.print_program_log(self.shader_program);
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
			println!("GL: {}:{}", msg, log_str);
		}

		return buf_size;
	}

	fn print_program_log(&self, prog: u32) -> i32 {
		let mut buf_size: i32 = 0;

		unsafe {
			let mut link_status:i32 = 0;
			gl::GetProgramiv(prog, gl::LINK_STATUS, &mut link_status);
			if link_status == gl::FALSE.into() {
				println!("GL: Program not linked");
			}
			gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut buf_size);
		}
		if buf_size > 1 {
			let mut length = 0_i32;
			buf_size += 1;
			let mut log:Vec<u8> = Vec::with_capacity(1024);
			unsafe {
				gl::GetProgramInfoLog(prog, 1024, &mut length, log.as_mut_ptr().cast());
				log.set_len(length as usize);
			}
			let log_str = String::from_utf8_lossy(&log);
			println!("GL prog: {}", log_str);
		}

		return buf_size;
	}

	fn check_gl_error(&self, line: u32) {
		let err: u32;
		unsafe {
			err = gl::GetError();
		}
		match err {
			gl::NO_ERROR => {}
			gl::INVALID_ENUM => {
				println!("GL_INVALID_ENUM:{}", line);
			}
			gl::INVALID_VALUE => {
				println!("GL_INVALID_VALUE:{}", line);
			}
			gl::INVALID_OPERATION => {
				println!("GL_INVALID_OPERATION:{}", line);
			}
			gl::INVALID_FRAMEBUFFER_OPERATION => {
				println!("GL_INVALID_FRAMEBUFFER_OPERATION:{}", line);
			}
			gl::OUT_OF_MEMORY => {
				println!("GL_OUT_OF_MEMORY:{}", line);
			}
			_ => {
				println!("GL: unknown error:{}:{}", line, err);
			}
		}
	}
}
