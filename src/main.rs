extern crate nalgebra_glm as glm;
// use gl::types::*;

use std::f32::consts::PI;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::{mem, os::raw::c_void, ptr};

mod mesh;
mod scene_graph;
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
use scene_graph::SceneNode;

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
unsafe fn create_vao(
    vertices: &Vec<f32>,
    colors: &Vec<f32>,
    normals: &Vec<f32>,
    indices: &Vec<u32>,
    mut vao: gl::types::GLuint,
) -> u32 {
    // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
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
        for j in 0..3 {
            data.push(normals[3 * i + j])
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
        (10 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        std::ptr::null(),                                      // offset of the first component
    );
    gl::EnableVertexAttribArray(1); // this is "layout (location = 1)" in vertex shader
    gl::VertexAttribPointer(
        1,         // index of the generic vertex attribute ("layout (location = 0)")
        4,         // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (10 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        offset::<f32>(3),                                      // offset of the first component
    );

    gl::EnableVertexAttribArray(2); // this is "layout (location = 1)" in vertex shader
    gl::VertexAttribPointer(
        2,         // index of the generic vertex attribute ("layout (location = 0)")
        4,         // the number of components per generic vertex attribute
        gl::FLOAT, // data type
        gl::FALSE, // normalized (int-to-float conversion)
        (10 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
        offset::<f32>(7),                                      // offset of the first component
    );
    let mut ibo: gl::types::GLuint = vao;
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
        .set_cursor_grab(true)
        .expect("failed to grab cursor");
    windowed_context.window().set_cursor_visible(false);
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

        let lunar_program_id: gl::types::GLuint;
        let heli_program_id: gl::types::GLuint;
        let lunar_vao: gl::types::GLuint = 1;
        let heli_vao: gl::types::GLuint = 2;
        let main_rotor_vao: gl::types::GLuint = 3;
        let tail_rotor_vao: gl::types::GLuint = 4;
        let door_vao: gl::types::GLuint = 5;
        let lunar_surface = Terrain::load("resources\\lunarsurface.obj");
        let helicopter = Helicopter::load("resources\\helicopter.obj");
        unsafe {
            //create vaos
            //reate the vao
            create_vao(
                &lunar_surface.vertices,
                &lunar_surface.colors,
                &lunar_surface.normals,
                &lunar_surface.indices,
                lunar_vao,
            );
            create_vao(
                &helicopter.body.vertices,
                &helicopter.body.colors,
                &helicopter.body.normals,
                &helicopter.body.indices,
                heli_vao,
            );
            create_vao(
                &helicopter.main_rotor.vertices,
                &helicopter.main_rotor.colors,
                &helicopter.main_rotor.normals,
                &helicopter.main_rotor.indices,
                main_rotor_vao,
            );
            create_vao(
                &helicopter.tail_rotor.vertices,
                &helicopter.tail_rotor.colors,
                &helicopter.tail_rotor.normals,
                &helicopter.tail_rotor.indices,
                tail_rotor_vao,
            );
            create_vao(
                &helicopter.door.vertices,
                &helicopter.door.colors,
                &helicopter.door.normals,
                &helicopter.door.indices,
                door_vao,
            );
            gl::BindVertexArray(lunar_vao);
            //I personally think this was way to difficult to figure out...
            let shader_builder = shader::ShaderBuilder::new();
            let shader_builder = shader_builder.attach_file("shaders\\simple.vert");
            let shader_builder = shader_builder.attach_file("shaders\\simple.frag");
            let shader_builder = shader_builder.link();
            lunar_program_id = shader_builder.program_id;
            gl::UseProgram(lunar_program_id);

            // gl::BindVertexArray(vao);
            // //I personally think this was way to difficult to figure out...
            // let shader_builder = shader::ShaderBuilder::new();
            // let shader_builder = shader_builder.attach_file("shaders\\simple.vert");
            // let shader_builder = shader_builder.attach_file("shaders\\simple.frag");
            // let shader_builder = shader_builder.link();
            // heli_program_id = shader_builder.program_id;
            // gl::UseProgram(heli_program_id);
        }

        // Used to demonstrate keyboard handling -- feel free to remove
        let movement_spd = 100.;
        let camera_spd = 1.;
        let mut last_frame_time = std::time::Instant::now();
        let first_frame_time = std::time::Instant::now();
        //The perspective matrix of the camera
        let camer_intrinsic_matrix: glm::Mat4 = glm::perspective(1., PI / 2., 0.1, 50000.);
        //The Translation matrix, used to store the current translation of the camera
        let mut camera_translation_matrix: glm::Mat4 = glm::translation(&glm::vec3(0.0, 0.0, 0.0));
        //The Rotation matrix, used to store the current translation of the camera
        let mut camera_rotation_matrix: glm::Mat4 = glm::rotation(0., &glm::vec3(1.0, 0.0, 0.0));

        //The final camera matrix, used to combine the other matricies
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            last_frame_time = now;

            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                let step = delta_time * movement_spd;

                // Used to get a more natural movement of the camera
                let dirz = glm::inverse(&camera_rotation_matrix) * glm::vec4(0., 0., 1., 1.);
                let dirz = step * glm::normalize(&glm::vec3(dirz[0], 0., dirz[2]));

                let dirx = glm::inverse(&camera_rotation_matrix) * glm::vec4(1., 0., 0., 1.);
                let dirx = step * glm::normalize(&glm::vec3(dirx[0], 0., dirx[2]));

                let diry = glm::vec3(0., step, 0.);
                for key in keys.iter() {
                    match key {
                        VirtualKeyCode::W => {
                            camera_translation_matrix =
                                glm::translation(&dirz) * camera_translation_matrix
                        }
                        VirtualKeyCode::S => {
                            camera_translation_matrix =
                                glm::translation(&-dirz) * camera_translation_matrix
                        }

                        VirtualKeyCode::A => {
                            camera_translation_matrix =
                                glm::translation(&dirx) * camera_translation_matrix
                        }
                        VirtualKeyCode::D => {
                            camera_translation_matrix =
                                glm::translation(&-dirx) * camera_translation_matrix
                        }

                        VirtualKeyCode::Q => {
                            camera_translation_matrix =
                                glm::translation(&diry) * camera_translation_matrix
                        }
                        VirtualKeyCode::E => {
                            camera_translation_matrix =
                                glm::translation(&-diry) * camera_translation_matrix
                        }
                        _ => {}
                    }
                }
            }
            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                let yaw_delta = (*delta).0 * 0.001 * camera_spd;
                let pitch_dleta = -(*delta).1 * 0.001 * camera_spd;

                // this one was wrong in the previous assignment
                camera_rotation_matrix = glm::rotation(pitch_dleta, &glm::vec3(-1.0, 0.0, 0.0))
                    * camera_rotation_matrix
                    * glm::rotation(yaw_delta, &glm::vec3(0.0, 1.0, 0.0));
                // rotation = glm::rotation(yaw, &glm::vec3(-1.0, 0.0, 0.0)) * rotation;
                *delta = (0.0, 0.0);
                // println!["{:?}", glm::rotation(0., &glm::vec3(1.0, 0.0, 0.0))]
            }
            let mut root_scene = SceneNode::new();
            let mut lunar_scene = SceneNode::from_vao(lunar_vao, lunar_surface.index_count);
            let mut heli_scene = SceneNode::from_vao(heli_vao, helicopter.body.index_count);
            let mut main_rotor_scene =
                SceneNode::from_vao(main_rotor_vao, helicopter.main_rotor.index_count);
            let mut tail_rotor_scene =
                SceneNode::from_vao(tail_rotor_vao, helicopter.tail_rotor.index_count);
            let door_scene = SceneNode::from_vao(door_vao, helicopter.door.index_count);

            root_scene.add_child(&lunar_scene);
            root_scene.add_child(&heli_scene);
            heli_scene.add_child(&main_rotor_scene);
            heli_scene.add_child(&tail_rotor_scene);
            heli_scene.add_child(&door_scene);

            let lightsource = glm::vec3::<f32>(3000. * (0. * elapsed).cos(), 1000., 0.);
            let view_projection_matrix =
                camer_intrinsic_matrix * camera_rotation_matrix * camera_translation_matrix;
            unsafe {
                gl::ClearColor(0.163, 0.163, 0.163, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                let cname = CString::new("ViewProjectionMatrix")
                    .expect("expected uniform name to have no nul bytes");
                let unilocation = gl::GetUniformLocation(
                    lunar_program_id,
                    cname.as_bytes_with_nul().as_ptr() as *const i8,
                );
                gl::UniformMatrix4fv(
                    unilocation,
                    1,
                    gl::FALSE,
                    view_projection_matrix.as_slice().as_ptr() as *const f32,
                );

                let cname = CString::new("CameraPosition")
                    .expect("expected uniform name to have no nul bytes");
                let unilocation = gl::GetUniformLocation(
                    lunar_program_id,
                    cname.as_bytes_with_nul().as_ptr() as *const i8,
                );
                gl::Uniform3fv(
                    unilocation,
                    1,
                    glm::vec4_to_vec3(
                        &(glm::inverse(&camera_translation_matrix) * glm::vec4(0., 0., 0., 1.)),
                    )
                    .as_ptr() as *const f32,
                );
                println![
                    "{:?}",
                    glm::vec4_to_vec3(
                        &(glm::inverse(&camera_translation_matrix) * glm::vec4(0., 0., 0., 1.))
                    )
                ];
                for vao in vec![
                    lunar_vao,
                    heli_vao,
                    main_rotor_vao,
                    tail_rotor_vao,
                    door_vao,
                ] {
                    gl::BindVertexArray(vao);
                    gl::DrawElements(
                        gl::TRIANGLES,
                        lunar_surface.index_count,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                    );
                }

                let cname = CString::new("LightSource")
                    .expect("expected uniform name to have no nul bytes");
                let unilocation = gl::GetUniformLocation(
                    lunar_program_id,
                    cname.as_bytes_with_nul().as_ptr() as *const i8,
                );
                gl::Uniform3fv(unilocation, 1, lightsource.as_ptr() as *const f32);
                for vao in vec![
                    lunar_vao,
                    heli_vao,
                    main_rotor_vao,
                    tail_rotor_vao,
                    door_vao,
                ] {
                    gl::BindVertexArray(vao);
                    gl::DrawElements(
                        gl::TRIANGLES,
                        lunar_surface.index_count,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                    );
                }
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
