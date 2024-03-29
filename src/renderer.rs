extern crate gl;
extern crate sdl2;

use crate::io::*;
use gl::types::GLuint;
use sdl2::audio::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLContext;
use sdl2::video::GLProfile;
use sdl2::video::Window;
use sdl2::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::string::String;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct AudioRenderer {
    io: Arc<Mutex<IO>>,
}

pub struct Renderer {
    io: Arc<Mutex<IO>>,
    vbr: Arc<(Mutex<VBR>, Condvar)>,
    tex_id: u32,
    vao: u32,
    vbo: u32,
    tex_data: Vec<u8>,
    sdl_context: Sdl,
    shader_program: u32,
    window: Window,
    audio_device: AudioDevice<AudioRenderer>,
    gl_context: GLContext,

    window_width: u32,
    window_height: u32,
}

impl AudioCallback for AudioRenderer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut io = self.io.lock().unwrap();
        io.read_audio(out);
    }
}

#[derive(Eq)]
struct GLVersion {
    majaor: u8,
    minor: u8,
}

impl GLVersion {
    pub fn new(s: &str) -> GLVersion {
        let v: Vec<&str> = s.split(".").collect();
        GLVersion {
            majaor: v[0].parse().unwrap(),
            minor: v[1].parse().unwrap(),
        }
    }
}

impl PartialOrd for GLVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GLVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.majaor > other.majaor {
            Ordering::Greater
        } else if self.majaor < other.majaor {
            Ordering::Less
        } else {
            if self.minor > other.minor {
                Ordering::Greater
            } else if self.minor < other.minor {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    }
}

impl PartialEq for GLVersion {
    fn eq(&self, other: &Self) -> bool {
        self.majaor == other.majaor && self.minor == other.minor
    }
}

impl Renderer {
    pub fn new(io: Arc<Mutex<IO>>, vbr: Arc<(Mutex<VBR>, Condvar)>) -> Renderer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();
        let controller_subsystem = sdl_context.game_controller().unwrap();

        let nr_controller = controller_subsystem.num_joysticks().unwrap();
        //println!("Number of controllers: {}", nr_controller);

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(2, 1);

        let window = video_subsystem
            .window("NES", 256, 240)
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);

        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (2, 1));

        let d_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            //samples: Some(735) // 44100/60
            samples: Some(1024),
        };
        let audio_device = audio_subsystem
            .open_playback(None, &d_spec, |spec| AudioRenderer {
                io: Arc::clone(&io),
            })
            .unwrap();

        let mut ret = Renderer {
            io: io,
            vbr: vbr,
            tex_id: 0,
            vao: 0,
            vbo: 0,
            tex_data: vec![0; 256 * 240 * 3],
            shader_program: 0,
            sdl_context: sdl_context,
            gl_context: ctx,
            window: window,
            audio_device: audio_device,

            window_width: 0,
            window_height: 0,
        };

        ret.init_gl();

        return ret;
    }

    pub fn event_loop(&mut self) {
        self.audio_device.resume();
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut first_time: bool = true;
        'running: loop {
            self.check_gl_error(line!());
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.8, 1.0);
                self.check_gl_error(line!());
                gl::Clear(gl::COLOR_BUFFER_BIT);
                self.check_gl_error(line!());
            }

            self.check_gl_error(line!());
            {
                let io = self.io.lock().unwrap();
                self.tex_data[0..].copy_from_slice(&io.vram[0..]);
            }

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);

                gl::BindTexture(gl::TEXTURE_2D, self.tex_id);
                self.check_gl_error(line!());
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGB as i32,
                    256,
                    240,
                    0,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                    self.tex_data.as_ptr() as *const c_void,
                );
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
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    Event::KeyDown {
                        keycode: Some(Keycode::X),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_a(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::X),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_a(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Z),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_b(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Z),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_b(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_start(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_start(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::RShift),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_select(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::RShift),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_select(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_up(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_up(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_down(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_down(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_left(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_left(0, 0);
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_right(0, 1);
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        self.io.lock().unwrap().pad.set_right(0, 0);
                    }

                    Event::Window {
                        win_event: Resized, ..
                    } => {
                        self.window_resized();
                    }
                    _ => {}
                }
            }

            let (vbr, cond) = &*self.vbr;
            cond.notify_all();
        }
    }

    fn window_resized(&mut self) {
        let (w, h) = self.window.size();
        if w != self.window_width && h != self.window_height {
            self.window_width = w;
            self.window_height = h;
            unsafe {
                gl::Viewport(0, 0, w as i32, h as i32);
                self.check_gl_error(line!());
            }
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

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                256,
                240,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                self.tex_data.as_ptr() as *const c_void,
            );
            self.check_gl_error(line!());

            // create object
            let position_data: Vec<f32> = vec![
                //  x     y     z    u    v
                -1.0, 1.0, 0.0, 0.0, 0.0, -1.0, -1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0,
                -1.0, 0.0, 1.0, 1.0,
            ];
            gl::GenVertexArrays(1, &mut self.vao);
            self.check_gl_error(line!());
            gl::BindVertexArray(self.vao);
            self.check_gl_error(line!());

            gl::GenBuffers(1, &mut self.vbo);
            self.check_gl_error(line!());
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            self.check_gl_error(line!());
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<f32>() * 20) as isize,
                position_data.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            self.check_gl_error(line!());

            gl::EnableVertexAttribArray(0);
            self.check_gl_error(line!());
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 4 * 5, 0 as *const c_void);
            self.check_gl_error(line!());
            gl::EnableVertexAttribArray(1);
            self.check_gl_error(line!());
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 5, 12 as *const c_void);
            self.check_gl_error(line!());

            gl::BindVertexArray(0);
            self.check_gl_error(line!());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            self.check_gl_error(line!());
        }
        self.create_program();
    }

    fn create_program(&mut self) {
        let vs_src_120: CString = CString::new(
            "
			#version 120

            attribute vec2 tex_uv;
            varying vec2 tex_coord;

			void main() {
				gl_Position = gl_Vertex;
                tex_coord = tex_uv;
			}
		",
        )
        .unwrap();
        let fs_src_120: CString = CString::new(
            "
			#version 120

            uniform sampler2D image;
            varying vec2 tex_coord;
			
			void main() {
                gl_FragColor = texture2D(image, tex_coord);
			}
		",
        )
        .unwrap();
        let vs_src_150: CString = CString::new(
            "
			#version 150 core
			in vec4 position;
			out vec2 texcoord;
 
			void main() {
				gl_Position = position;
				texcoord = vec2(gl_VertexID/2, gl_VertexID%2);
			}
		",
        )
        .unwrap();
        let fs_src_150: CString = CString::new(
            "
			#version 150 core
                       
			uniform sampler2D image;
			in vec2 texcoord;
			out vec4 fragment;
 
			void main() {
				fragment = texture(image, texcoord);
			}
		",
        )
        .unwrap();
        let vs_src: &CString;
        let fs_src: &CString;

        unsafe {
            self.shader_program = gl::CreateProgram();
        }

        let mut glsl_version: GLVersion;
        unsafe {
            let v = gl::GetString(gl::SHADING_LANGUAGE_VERSION);
            let cstr = CStr::from_ptr(v as *const i8);
            println!("SHADING_LANGUAGE_VERSION:{}", cstr.to_str().unwrap());
            glsl_version = GLVersion::new(cstr.to_str().unwrap());
            let version_120 = GLVersion::new("1.20");
            if glsl_version > version_120 {
                vs_src = &vs_src_150;
                fs_src = &fs_src_150;
            } else {
                vs_src = &vs_src_120;
                fs_src = &fs_src_120;
            }
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
            gl::BindAttribLocation(
                self.shader_program,
                0,
                CString::new("position").unwrap().as_ptr(),
            );
            self.check_gl_error(line!());
            gl::BindAttribLocation(
                self.shader_program,
                1,
                CString::new("tex_uv").unwrap().as_ptr(),
            );
            self.check_gl_error(line!());
            gl::BindFragDataLocation(
                self.shader_program,
                0,
                CString::new("fragment").unwrap().as_ptr(),
            );
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
            let mut log: Vec<u8> = vec![0; buf_size as usize];
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
            let mut link_status: i32 = 0;
            gl::GetProgramiv(prog, gl::LINK_STATUS, &mut link_status);
            if link_status == gl::FALSE.into() {
                println!("GL: Program not linked");
            }
            gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut buf_size);
        }
        if buf_size > 1 {
            let mut length = 0_i32;
            buf_size += 1;
            let mut log: Vec<u8> = Vec::with_capacity(1024);
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
