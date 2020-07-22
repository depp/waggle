use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile};
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::ptr::null;
use std::slice;
use std::str;
use takeable_option::Takeable;

use gl;
use gl::types::*;

pub fn main() {
    let (raw_context, el) = {
        let el = EventLoop::new();
        let wb = WindowBuilder::new()
            .with_title("A fantastic window!")
            .with_inner_size(PhysicalSize::new(2200, 512));

        let raw_context = ContextBuilder::new()
            .with_gl_profile(GlProfile::Core)
            .with_gl_debug_flag(true)
            .build_windowed(wb, &el)
            .unwrap();

        (raw_context, el)
    };
    let raw_context = unsafe { raw_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        raw_context.get_pixel_format()
    );

    gl::load_with(|name| raw_context.get_proc_address(name));
    assert!(gl::ClearColor::is_loaded());
    assert!(gl::Clear::is_loaded());
    eprintln!(
        "OpenGL Version: {}",
        unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const c_char) }
            .to_str()
            .unwrap()
    );
    let mut has_khr_debug = false;
    for ext in extensions() {
        match ext {
            "GL_KHR_debug" => has_khr_debug = true,
            _ => (),
        }
    }
    let mut flags = 0;
    unsafe { gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags) };
    if has_khr_debug && (flags as u32) & gl::CONTEXT_FLAG_DEBUG_BIT != 0 {
        unsafe { gl::DebugMessageCallbackKHR(Some(debug_callback), null()) };
    }

    let app = Application::new();
    let mut degree = 0.0;
    let mut x: i32 = 2200;
    let mut y: i32 = 512;

    let mut raw_context = Takeable::new(raw_context);
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                //raw_context.window().request_redraw();
            }
            Event::LoopDestroyed => {
                Takeable::take(&mut raw_context); // Make sure it drops first
                return;
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    raw_context.resize(physical_size);
                    unsafe {
                        gl::Viewport(
                            0,
                            0,
                            physical_size.width as i32,
                            physical_size.height as i32,
                        );
                    }
                    x = physical_size.width as i32;
                    y = physical_size.height as i32;
                    println!("sizes: {}, {}", x, y);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                println!("el {:?}", event);
                app.render(x, y);
                raw_context.swap_buffers().unwrap();
                degree += 1.0f32;
            }
            _ => (),
        }
    });
}

struct Extensions(GLint, GLint);

fn extensions() -> Extensions {
    let mut count = 0;
    unsafe { gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut count) };
    Extensions(0, count)
}

impl Iterator for Extensions {
    type Item = &'static str;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut Extensions(ref mut index, count) = self;
        if *index < count {
            let name = unsafe {
                CStr::from_ptr(gl::GetStringi(gl::EXTENSIONS, *index as GLuint) as *const c_char)
            }
            .to_str()
            .unwrap();
            *index += 1;
            Some(name)
        } else {
            None
        }
    }
}

extern "system" fn debug_callback(
    source: GLenum,
    gltype: GLenum,
    id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    let source = match source {
        gl::DEBUG_SOURCE_API => "api",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "shader_compiler",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "window_system",
        gl::DEBUG_SOURCE_THIRD_PARTY => "third_party",
        gl::DEBUG_SOURCE_APPLICATION => "application",
        gl::DEBUG_SOURCE_OTHER => "other",
        _ => "unknown",
    };
    let gltype = match gltype {
        gl::DEBUG_TYPE_ERROR => "error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated_behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined_behavior",
        gl::DEBUG_TYPE_PERFORMANCE => "performance",
        gl::DEBUG_TYPE_PORTABILITY => "portability",
        gl::DEBUG_TYPE_OTHER => "other",
        gl::DEBUG_TYPE_MARKER => "marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "push_group",
        gl::DEBUG_TYPE_POP_GROUP => "pop_group",
        _ => "unknown",
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "high",
        gl::DEBUG_SEVERITY_MEDIUM => "medium",
        gl::DEBUG_SEVERITY_LOW => "low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "notification",
        _ => "unknown",
    };
    let message =
        str::from_utf8(unsafe { slice::from_raw_parts(message as *const u8, length as usize) })
            .unwrap();
    eprintln!(
        "OpenGL: src={}, type={}, id={}, sev={}: {}",
        source, gltype, id, severity, message
    );
}

const VERTEX_SRC: &str = include_str!("vertex_shader.glsl");
const FRAGMENT_SRC: &str = include_str!("fragment_shader.glsl");
const TRI_VERTS: [GLfloat; 6] = [40.0f32, 16.0f32, 104.0f32, 16.0f32, 104.0f32, 80.0f32];
const TRI_COLS: [GLfloat; 12] = [
    0.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32, 0.0f32, 0.0f32,
];

fn init_shaders() -> GLuint {
    unsafe {
        let vshader = gl::CreateShader(gl::VERTEX_SHADER);
        let vsrc_ptr: &CStr = &CString::new(VERTEX_SRC).unwrap();
        gl::ShaderSource(vshader, 1, &vsrc_ptr.as_ptr(), std::ptr::null());
        gl::CompileShader(vshader);
        let mut compiled = 0;
        gl::GetShaderiv(vshader, gl::COMPILE_STATUS, &mut compiled);
        println!("vshader compilation: {}", compiled);
        if compiled == 0 {
            let mut info_len = 0;
            gl::GetShaderiv(vshader, gl::INFO_LOG_LENGTH, &mut info_len);
            if info_len > 0 {
                let mut buf: Vec<i8> = std::iter::repeat(0).take((info_len) as usize).collect();
                gl::GetShaderInfoLog(vshader, info_len, std::ptr::null_mut(), buf.as_mut_ptr());
                let ubuf: Vec<u8> = buf
                    .iter()
                    .map(|&x| x as u8)
                    .take_while(|&x| x != 0)
                    .collect();
                println!(
                    "Could not compile shader: {}",
                    CString::new(ubuf).unwrap().to_str().unwrap()
                );
            }
        }

        let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let fsrc_ptr: &CStr = &CString::new(FRAGMENT_SRC).unwrap();
        gl::ShaderSource(fshader, 1, &fsrc_ptr.as_ptr(), std::ptr::null());
        gl::CompileShader(fshader);
        let mut compiled = 0;
        gl::GetShaderiv(fshader, gl::COMPILE_STATUS, &mut compiled);
        println!("fshader compilation: {}", compiled);
        if compiled == 0 {
            let mut info_len = 0;
            gl::GetShaderiv(fshader, gl::INFO_LOG_LENGTH, &mut info_len);
            if info_len > 0 {
                let mut buf: Vec<i8> = std::iter::repeat(0).take((info_len) as usize).collect();
                gl::GetShaderInfoLog(fshader, info_len, std::ptr::null_mut(), buf.as_mut_ptr());
                let ubuf: Vec<u8> = buf
                    .iter()
                    .map(|&x| x as u8)
                    .take_while(|&x| x != 0)
                    .collect();
                println!(
                    "Could not compile shader: {}",
                    CString::new(ubuf).unwrap().to_str().unwrap()
                );
            }
        }
        println!("shaders: {}, {}", vshader, fshader);
        // currently assuming this compiles correctly

        let program = gl::CreateProgram();
        gl::AttachShader(program, vshader);
        gl::AttachShader(program, fshader);
        gl::LinkProgram(program);
        return program;
    }
}

// this is Some Shenanigans to deal with std140 layouts - next time use the shader_types crate
const BITSTRING: [[u32; 4]; 4] = [
    [0xAAAAAAAA; 4],
    [0x55555555; 4],
    [0x0000FFFF; 4],
    [0x1425e1fe; 4],
];

struct Application {
    program: GLuint,      // Program
    array: GLuint,        // Array
    block: GLuint,        // Buffer
    dims: GLint,          // Uniform
    block_handle: GLuint, // UBO index
}

impl Application {
    fn new() -> Self {
        let program = init_shaders();
        unsafe {
            let pos_handle: GLuint =
                gl::GetAttribLocation(program, CString::new("VertexPosition").unwrap().as_ptr())
                    as GLuint;
            let mut array = 0;
            let mut buffers = [0; 2];
            gl::GenVertexArrays(1, &mut array);
            gl::GenBuffers(2, buffers.as_mut_ptr());
            let [verts, block] = buffers;
            gl::BindVertexArray(array);
            gl::BindBuffer(gl::ARRAY_BUFFER, verts);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&TRI_VERTS) as GLsizeiptr,
                TRI_VERTS.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(pos_handle);
            gl::VertexAttribPointer(pos_handle, 2, gl::FLOAT, 0, 0, null());
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            let dims =
                gl::GetUniformLocation(program, CString::new("Dimensions").unwrap().as_ptr());
            let block_handle =
                gl::GetUniformBlockIndex(program, CString::new("bitstring").unwrap().as_ptr());
            Application {
                program,
                array,
                block,
                dims,
                block_handle,
            }
        }
    }

    fn render(&self, x: i32, y: i32) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);

            gl::Uniform2f(self.dims, x as f32, y as f32);

            gl::BindBuffer(gl::UNIFORM_BUFFER, self.block);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                16 * 4,
                BITSTRING.as_ptr() as *const c_void,
                gl::DYNAMIC_DRAW,
            );
            gl::UniformBlockBinding(self.program, self.block_handle, 2);
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 2, self.block);

            gl::BindVertexArray(self.array);

            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 3, 4 * 32);
        }
    }
}
