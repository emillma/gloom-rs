extern crate nalgebra_glm as glm;
// use gl::types::*;

use std::f32::consts::PI;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod mesh;
mod shader;
mod triangles;
mod util;
use triangles::get_triangles;

use glutin::event::{
    DeviceEvent,
    ElementState::{Pressed, Released},
    Event, KeyboardInput,
    VirtualKeyCode::{self, *},
    WindowEvent,
};
use glutin::event_loop::ControlFlow;
use std::ffi::CString;

use mesh::{Helicopter, Terrain};
const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 800;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //
// The names should be pretty self explanatory
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
#[allow(dead_code)]
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
#[allow(dead_code)]
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T
#[allow(dead_code)]
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()

// == // Modify and complete the function below for the first task
unsafe fn create_vao(vertices: &Vec<f32>, colors: &Vec<f32>, indices: &Vec<u32>) -> u32 {
    // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    let mut vao: gl::types::GLuint = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    let mut data: Vec<f32> = Vec::new();
    for i in 0..vertices.len() / 3 {
        for j in 0..3 {
            data.push(vertices[3 * i + j])
        }

        for j in 0..4 {
            data.push(colors[4 * i + j])
        }
    }

    let mut vbo: gl::types::GLuint = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    gl::BufferData(
        gl::ARRAY_BUFFER,                          // target
        byte_size_of_array(&data),                 // size of data in bytes
        data.as_ptr() as *const gl::types::GLvoid, // pointer to data
        gl::STATIC_DRAW,                           // usage
    );

    gl::EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
    gl::VertexAttribPointer(
        0,         // index of the generic vertex attribute ("layout (location = 0)")
        3,         // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (7 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        std::ptr::null(),                                     // offset of the first component
    );
    gl::EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
    gl::VertexAttribPointer(
        1,         // index of the generic vertex attribute ("layout (location = 0)")
        4,         // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (7 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        offset::<f32>(3),                                     // offset of the first component
    );
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
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    windowed_context
        .window()
        .set_cursor_grab(false)
        .expect("failed to grab cursor");
    windowed_context.window().set_cursor_visible(true);
    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };
        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!(
                "{}: {}",
                util::get_gl_string(gl::VENDOR),
                util::get_gl_string(gl::RENDERER)
            );
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!(
                "GLSL\t: {}",
                util::get_gl_string(gl::SHADING_LANGUAGE_VERSION)
            );
        }

        //set up verticies
        let n: u32 = 3;

        let unilocation: gl::types::GLint;
        let lunar_surface = Terrain::load("resources\\lunarsurface.obj");
        unsafe {
            //reate the vao
            let vao = create_vao(
                &lunar_surface.vertices,
                &lunar_surface.colors,
                &lunar_surface.indices,
            );
            gl::BindVertexArray(vao);
            //I personally think this was way to difficult to figure out...
            let shader_builder = shader::ShaderBuilder::new();
            let shader_builder = shader_builder.attach_file("shaders\\simple.vert");
            let shader_builder = shader_builder.attach_file("shaders\\simple.frag");
            let shader_builder = shader_builder.link();

            let cname =
                CString::new("ViewProjection").expect("expected uniform name to have no nul bytes");
            unilocation = gl::GetUniformLocation(
                shader_builder.program_id,
                cname.as_bytes_with_nul().as_ptr() as *const i8,
            );
            println!("unilocation {:?}", unilocation);

            gl::UseProgram(shader_builder.program_id);
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let movement_spd = 1.;
        let camera_spd = 1.;
        let mut last_frame_time = std::time::Instant::now();

        //The perspective matrix of the camera
        let perspective: glm::Mat4 = glm::perspective(1., PI / 2., 0.1, 100.);
        //The Translation matrix, used to store the current translation of the camera
        let mut translation: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, 0.0));
        //The Rotation matrix, used to store the current translation of the camera
        let mut rotation: glm::Mat4 = glm::rotation(0., &glm::vec3(1.0, 0.0, 0.0));

        //The final camera matrix, used to combine the other matricies
        let mut camera_matrix: glm::Mat4;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                let step = delta_time * movement_spd * 3.;

                // Used to get a more natural movement of the camera
                let dirz = step * glm::inverse(&rotation) * glm::vec4(0., 0., 1., 1.);
                let dirz = glm::vec3(dirz[0], dirz[1], dirz[2]);
                let dirx = step * glm::inverse(&rotation) * glm::vec4(1., 0., 0., 1.);
                let dirx = glm::vec3(dirx[0], dirx[1], dirx[2]);
                let diry = step * glm::inverse(&rotation) * glm::vec4(0., 1., 0., 1.);
                let diry = glm::vec3(diry[0], diry[1], diry[2]);
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => translation = glm::translation(&dirz) * translation,
                        VirtualKeyCode::S => translation = glm::translation(&-dirz) * translation,

                        VirtualKeyCode::A => translation = glm::translation(&dirx) * translation,
                        VirtualKeyCode::D => translation = glm::translation(&-dirx) * translation,

                        VirtualKeyCode::Q => translation = glm::translation(&diry) * translation,
                        VirtualKeyCode::E => translation = glm::translation(&-diry) * translation,
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                let x = (*delta).0 * 0.001 * camera_spd;
                let y = -(*delta).1 * 0.001 * camera_spd;
                rotation = glm::rotation(x, &glm::vec3(0.0, 1.0, 0.0)) * rotation;
                rotation = glm::rotation(y, &glm::vec3(-1.0, 0.0, 0.0)) * rotation;
                *delta = (0.0, 0.0);
                // println!["{:?}", glm::rotation(0., &glm::vec3(1.0, 0.0, 0.0))]
            }
            camera_matrix = perspective * rotation * translation;
            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::UniformMatrix4fv(
                    unilocation,
                    1,
                    gl::FALSE,
                    camera_matrix.as_slice().as_ptr() as *const f32,
                );
                gl::DrawElements(
                    gl::TRIANGLES,
                    lunar_surface.indices.len() as i32,
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
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => {}
        }
    });
}
