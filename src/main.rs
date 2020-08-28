extern crate nalgebra_glm as glm;
use gl::types::*;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr, str};

mod shader;
mod util;

use glutin::event::{
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 800;

// Helper functions to make interacting with OpenGL a little bit prettier. You will need these!
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// == // Modify and complete the function below for the first task
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    let mut vao: gl::types::GLuint = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    let mut vbo: gl::types::GLuint = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,                              // target
        byte_size_of_array(vertices),                  // size of data in bytes
        vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
        gl::STATIC_DRAW,                               // usage
    );
    gl::VertexAttribPointer(
        0,         // index of the generic vertex attribute ("layout (location = 0)")
        3,         // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        std::ptr::null(),                                     // offset of the first component
    );
    gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader

    let mut ibo: gl::types::GLuint = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,                     // target
        byte_size_of_array(indices),                  // size of data in bytes
        indices.as_ptr() as *const gl::types::GLvoid, // pointer to data
        gl::STATIC_DRAW,                              // usage
    );

    return vao;
}

fn main() {
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Send a copy of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the renderin thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        // Set up openGL
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());
        }
        //set up verticies
        let mut vertices: Vec<f32> = Vec::new();
        let n: u32 = 5;
        for i in 1..n + 1 {
            let i2 = i as f32;
            let size = 0.1;
            let offset = 0.9 - size * i2 * (i2 + 1.) / 2.;
            let triangle = vec![
                -size * i2,
                offset,
                0.,
                size * i2,
                offset,
                0.,
                0.,
                offset + size * i2,
                0.,
            ];
            vertices.extend_from_slice(&triangle);
        }
        //set up indices
        let indices: Vec<u32> = (0..(n * 3)).collect();
        // let vertices: Vec<f32> = vec![0.6, -0.8, -1.2, 0.0, 0.4, 0.0, -0.8, -0.2, 1.2];
        // let indices: Vec<u32> = vec![0, 2, 1];

        unsafe {
            //reate the vao
            let vao = create_vao(&vertices, &indices);
            gl::BindVertexArray(vao);

            //I personally think this was way to difficult to figure out...
            let shader_builder = shader::ShaderBuilder::new();
            let shader_builder = shader_builder.attach_file("shaders\\simple.vert");
            let shader_builder = shader_builder.attach_file("shaders\\simple.frag");
            let shader_builder = shader_builder.link();
            gl::UseProgram(shader_builder.program_id);
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let mut _arbitrary_number = 0.0;

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::A => {
                            _arbitrary_number += delta_time;
                        }
                        VirtualKeyCode::D => {
                            _arbitrary_number -= delta_time;
                        }

                        _ => {}
                    }
                }
            }

            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::DrawElements(
                    gl::TRIANGLES,
                    indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
                // Issue the necessary commands to draw your scene here
            }

            context.swap_buffers().unwrap();
        }
    });

    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events get handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: key_state,
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        }
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
