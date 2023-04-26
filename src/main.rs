use glfw::{Action, Context, Key};
use glow::HasContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw::WindowHint::ContextVersion(3, 3);
    glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core);
    glfw::WindowHint::OpenGlForwardCompat(true);

    let (mut window, events) = glfw.create_window(800, 600, "glow-repro", glfw::WindowMode::Windowed).expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    let gl = glow_context(&mut window);

    unsafe {
        gl.viewport(0, 0, 800, 600);

        let vertex_path = "shaders/grid_v.glsl";
        let frag_path = "shaders/grid_f.glsl";
        let mut vertex_shader_source = std::fs::read_to_string(vertex_path)?;
        vertex_shader_source.push('\0');

        let mut frag_shader_source = std::fs::read_to_string(frag_path)?;
        frag_shader_source.push('\0');

        let vertex_shader = gl.create_shader(glow::VERTEX_SHADER)?;
        gl.shader_source(vertex_shader, &vertex_shader_source);
        gl.compile_shader(vertex_shader);
        let vert_status = gl.get_shader_compile_status(vertex_shader);
        println!("vertex shader {:?} compiled with status: {}",
            std::path::Path::new(vertex_path).file_name().unwrap(),
            vert_status
        );

        let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER)?;
        gl.shader_source(frag_shader, &frag_shader_source);
        gl.compile_shader(frag_shader);
        let frag_status = gl.get_shader_compile_status(frag_shader);
        println!("frag shader {:?} compiled with status: {}",
            std::path::Path::new(frag_path).file_name().unwrap(),
            frag_status
        );

        let shader_program = gl.create_program()?;
        gl.attach_shader(shader_program, vertex_shader);
        gl.attach_shader(shader_program, frag_shader);
        gl.link_program(shader_program);
        gl.delete_shader(vertex_shader);
        gl.delete_shader(frag_shader);

        println!("shader program with id: {} was linked with status: {}", shader_program.0, gl.get_program_link_status(shader_program));

        while !window.should_close() {
            // camera matrices
            let view_mat = glm::ext::look_at(glm::vec3(0.0, 0.0, 0.0), glm::vec3(0.0, 0.0, -1.0), glm::vec3(0.0, 1.0, 0.0));
            let (win_width, win_height) = window.get_size();
            let projection_mat = glm::ext::perspective(glm::radians(45.0), win_width as f32 / win_height as f32, 0.1, 100.0);

            for (_, event) in glfw::flush_messages(&events) {
                handle_window_event(&mut window, &event, &gl);
            }

            gl.clear_color(0.3, 0.3, 0.3, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            gl.use_program(Some(shader_program));
            let loc1 = gl.get_uniform_location(shader_program, "view");
            let slice_val1 = std::slice::from_raw_parts(view_mat.as_array().as_ptr() as *const f32, 16);
            // we panic here when unwrapping loc1
            gl.uniform_matrix_4_f32_slice(Some(&loc1.unwrap()), false, slice_val1);

            let loc2 = gl.get_uniform_location(shader_program, "projection");
            let slice_val2 = std::slice::from_raw_parts(projection_mat.as_array().as_ptr() as *const f32, 16);
            gl.uniform_matrix_4_f32_slice(Some(&loc2.unwrap()), false, slice_val2);

            gl.draw_arrays(glow::TRIANGLES, 0, 6);

            glfw.poll_events();
            window.swap_buffers();
        }
    }

    Ok(())
}

fn handle_window_event(window: &mut glfw::Window, event: &glfw::WindowEvent, gl: &glow::Context) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }
        glfw::WindowEvent::FramebufferSize(width, height) => {
            unsafe {
                gl.viewport(0, 0, *width, *height);
            }
        }
        _ => {}
    }
}

fn glow_context(window: &mut glfw::Window) -> glow::Context {
    unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s).cast()) }
}

