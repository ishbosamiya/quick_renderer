extern crate image;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use quick_renderer::camera::WindowCamera;
use quick_renderer::drawable::Drawable;
use quick_renderer::egui;
use quick_renderer::egui_glfw;
use quick_renderer::fps::FPS;
use quick_renderer::framebuffer::FrameBuffer;
use quick_renderer::glfw;
use quick_renderer::glm;
use quick_renderer::gpu_immediate::GPUImmediate;
use quick_renderer::gpu_utils;
use quick_renderer::infinite_grid::InfiniteGrid;
use quick_renderer::infinite_grid::InfiniteGridDrawData;
use quick_renderer::jfa;
use quick_renderer::mesh;
use quick_renderer::mesh::{MeshDrawData, MeshUseShader};
use quick_renderer::renderbuffer::RenderBuffer;
use quick_renderer::shader;
use quick_renderer::texture::TextureRGBAFloat;

use std::convert::TryInto;

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // set to opengl 3.3 or higher
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // creating window
    let (mut window, events) = glfw
        .create_window(1280, 720, "Outline Render", glfw::WindowMode::Windowed)
        .expect("ERROR: glfw window creation failed");

    // setup bunch of polling data
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.set_char_polling(true);
    window.make_current();

    gl::load_with(|symbol| window.get_proc_address(symbol));

    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
    }

    // setup the egui backend
    let mut egui = EguiBackend::new(&mut window, &mut glfw);

    let mut fonts = FontDefinitions::default();
    // larger text
    fonts
        .family_and_size
        .insert(TextStyle::Button, (FontFamily::Proportional, 18.0));
    fonts
        .family_and_size
        .insert(TextStyle::Body, (FontFamily::Proportional, 18.0));
    fonts
        .family_and_size
        .insert(TextStyle::Small, (FontFamily::Proportional, 15.0));
    egui.get_egui_ctx().set_fonts(fonts);

    let mesh = mesh::builtins::get_monkey_subd_01();

    let mut camera = WindowCamera::new(
        glm::vec3(0.0, 0.0, 3.0),
        glm::vec3(0.0, 1.0, 0.0),
        -90.0,
        0.0,
        45.0,
    );

    let mut imm = GPUImmediate::new();

    let directional_light_shader = shader::builtins::get_directional_light_shader()
        .as_ref()
        .unwrap();

    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();

    let face_orientation_shader = shader::builtins::get_face_orientation_shader()
        .as_ref()
        .unwrap();

    let flat_texture_shader = shader::builtins::get_flat_texture_shader()
        .as_ref()
        .unwrap();

    let jfa_initialization_shader = shader::builtins::get_jfa_initialization_shader()
        .as_ref()
        .unwrap();

    let jfa_step_shader = shader::builtins::get_jfa_step_shader().as_ref().unwrap();

    let jfa_convert_to_distance_shader = shader::builtins::get_jfa_convert_to_distance_shader()
        .as_ref()
        .unwrap();

    println!(
        "directional_light: uniforms: {:?} attributes: {:?}",
        directional_light_shader.get_uniforms(),
        directional_light_shader.get_attributes(),
    );

    println!(
        "smooth_color_3d: uniforms: {:?} attributes: {:?}",
        smooth_color_3d_shader.get_uniforms(),
        smooth_color_3d_shader.get_attributes(),
    );

    println!(
        "face_orientation: uniforms: {:?} attributes: {:?}",
        face_orientation_shader.get_uniforms(),
        face_orientation_shader.get_attributes(),
    );

    println!(
        "flat_texture: uniforms: {:?} attributes: {:?}",
        flat_texture_shader.get_uniforms(),
        flat_texture_shader.get_attributes(),
    );

    println!(
        "jfa_initialization: uniforms: {:?} attributes: {:?}",
        jfa_initialization_shader.get_uniforms(),
        jfa_initialization_shader.get_attributes(),
    );

    println!(
        "jfa_step: uniforms: {:?} attributes: {:?}",
        jfa_step_shader.get_uniforms(),
        jfa_step_shader.get_attributes(),
    );

    println!(
        "jfa_convert_to_distance: uniforms: {:?} attributes: {:?}",
        jfa_convert_to_distance_shader.get_uniforms(),
        jfa_convert_to_distance_shader.get_attributes(),
    );

    let mut last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    let infinite_grid = InfiniteGrid::default();

    let mut jfa_num_steps = 0;
    let mut jfa_convert_to_distance = false;
    let mut test_image_resolution = (1920, 1080);

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(&event, &mut window, &mut camera, &mut last_cursor);
        });

        let (window_width, window_height) = window.get_size();
        let (window_width, window_height): (usize, usize) = (
            window_width.try_into().unwrap(),
            window_height.try_into().unwrap(),
        );

        let projection_matrix =
            &glm::convert(camera.get_projection_matrix(window_width, window_height));
        let view_matrix = &glm::convert(camera.get_view_matrix());

        // Shader stuff
        {
            {
                directional_light_shader.use_shader();
                directional_light_shader.set_mat4("projection\0", projection_matrix);
                directional_light_shader.set_mat4("view\0", view_matrix);
                directional_light_shader.set_mat4("model\0", &glm::identity());
                directional_light_shader
                    .set_vec3("viewPos\0", &glm::convert(camera.get_position()));
                directional_light_shader.set_vec3("material.color\0", &glm::vec3(0.3, 0.2, 0.7));
                directional_light_shader.set_vec3("material.specular\0", &glm::vec3(0.3, 0.3, 0.3));
                directional_light_shader.set_float("material.shininess\0", 4.0);
                directional_light_shader
                    .set_vec3("light.direction\0", &glm::vec3(-0.7, -1.0, -0.7));
                directional_light_shader.set_vec3("light.ambient\0", &glm::vec3(0.3, 0.3, 0.3));
                directional_light_shader.set_vec3("light.diffuse\0", &glm::vec3(1.0, 1.0, 1.0));
                directional_light_shader.set_vec3("light.specular\0", &glm::vec3(1.0, 1.0, 1.0));
            }

            {
                smooth_color_3d_shader.use_shader();
                smooth_color_3d_shader.set_mat4("projection\0", projection_matrix);
                smooth_color_3d_shader.set_mat4("view\0", view_matrix);
                smooth_color_3d_shader.set_mat4("model\0", &glm::identity());
            }

            {
                face_orientation_shader.use_shader();
                face_orientation_shader.set_mat4("projection\0", projection_matrix);
                face_orientation_shader.set_mat4("view\0", view_matrix);
                face_orientation_shader.set_mat4("model\0", &glm::identity());
                face_orientation_shader
                    .set_vec4("color_face_front\0", &glm::vec4(0.0, 0.0, 1.0, 1.0));
                face_orientation_shader
                    .set_vec4("color_face_back\0", &glm::vec4(1.0, 0.0, 0.0, 1.0));
            }

            {
                jfa_initialization_shader.use_shader();
            }

            {
                jfa_step_shader.use_shader();
            }

            {
                jfa_convert_to_distance_shader.use_shader();
            }

            {
                flat_texture_shader.use_shader();
                flat_texture_shader.set_mat4("projection\0", projection_matrix);
                flat_texture_shader.set_mat4("view\0", view_matrix);
                flat_texture_shader.set_mat4("model\0", &glm::identity());
            }
        }

        unsafe {
            gl::Disable(gl::BLEND);
        }

        FrameBuffer::activiate_default();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("model\0", &glm::identity());
        mesh.draw(&mut MeshDrawData::new(
            &mut imm,
            MeshUseShader::DirectionalLight,
            None,
        ))
        .unwrap();

        let mut test_image =
            TextureRGBAFloat::new_empty(test_image_resolution.0, test_image_resolution.1);
        // generate the test image by rendering a mesh
        {
            let mut prev_viewport_params = [0, 0, 0, 0];
            unsafe {
                gl::GetIntegerv(gl::VIEWPORT, prev_viewport_params.as_mut_ptr());
                gl::Viewport(
                    0,
                    0,
                    test_image.get_width().try_into().unwrap(),
                    test_image.get_height().try_into().unwrap(),
                );
            }
            let framebuffer = FrameBuffer::new();
            let renderbuffer = RenderBuffer::new(test_image_resolution.0, test_image_resolution.1);
            framebuffer.activate(&test_image, &renderbuffer);
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }

            smooth_color_3d_shader.use_shader();
            smooth_color_3d_shader.set_mat4(
                "projection\0",
                &glm::convert(
                    camera.get_projection_matrix(test_image.get_width(), test_image.get_height()),
                ),
            );
            smooth_color_3d_shader.set_mat4("model\0", &glm::identity());
            mesh.draw(&mut MeshDrawData::new(
                &mut imm,
                MeshUseShader::SmoothColor3D,
                Some(glm::vec4(1.0, 1.0, 1.0, 1.0)),
            ))
            .unwrap();

            // reset everything to what it was before this test image rendering
            FrameBuffer::activiate_default();
            unsafe {
                gl::Viewport(
                    prev_viewport_params[0],
                    prev_viewport_params[1],
                    prev_viewport_params[2],
                    prev_viewport_params[3],
                );
            }
            smooth_color_3d_shader.set_mat4("projection\0", projection_matrix);
        }

        {
            let mut jfa_texture = jfa::jfa(&mut test_image, jfa_num_steps, &mut imm);

            let mut final_texture;
            if jfa_convert_to_distance {
                final_texture = jfa::convert_to_distance(&mut jfa_texture, &mut imm);
            } else {
                final_texture = jfa_texture;
            }

            flat_texture_shader.use_shader();
            flat_texture_shader.set_int("image\0", 31);
            let final_texture_aspect_ratio =
                final_texture.get_width() as f32 / final_texture.get_height() as f32;
            flat_texture_shader.set_mat4(
                "model\0",
                &glm::scale(
                    &glm::translate(
                        &glm::identity(),
                        &glm::vec3(final_texture_aspect_ratio + 1.0, 1.0, 0.0),
                    ),
                    &glm::vec3(final_texture_aspect_ratio, 1.0, 1.0),
                ),
            );
            final_texture.activate(31);
            gpu_utils::render_quad(&mut imm, flat_texture_shader);
        }

        // Keep meshes that have shaders that need alpha channel
        // (blending) bellow this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            infinite_grid
                .draw(&mut InfiniteGridDrawData::new(
                    projection_matrix,
                    view_matrix,
                    &mut imm,
                ))
                .unwrap();
        }

        // GUI starts
        {
            egui.begin_frame(&window, &mut glfw);
            egui::Window::new("Hello world!").show(egui.get_egui_ctx(), |ui| {
                ui.label("Hello World, Outline Render!");
                ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));
                ui.add(
                    egui::Slider::new(&mut jfa_num_steps, 0..=30)
                        .text("JFA Num Steps")
                        .clamp_to_range(true),
                );
                ui.checkbox(&mut jfa_convert_to_distance, "JFA Convert To Distance");
                ui.add(
                    egui::Slider::new(&mut test_image_resolution.0, 1..=3840)
                        .text("Test Image Width")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut test_image_resolution.1, 1..=2160)
                        .text("Test Image Height")
                        .clamp_to_range(true),
                );
            });
            let _output = egui.end_frame(glm::vec2(window_width as _, window_height as _));
        }
        // GUI ends

        // Swap front and back buffers
        window.swap_buffers();
    }
}

fn handle_window_event(
    event: &glfw::WindowEvent,
    window: &mut glfw::Window,
    camera: &mut WindowCamera,
    last_cursor: &mut (f64, f64),
) {
    let cursor = window.get_cursor_pos();
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        }

        glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
            gl::Viewport(0, 0, *width, *height);
        },
        glfw::WindowEvent::Scroll(_, scroll_y) => {
            camera.zoom(*scroll_y);
        }
        _ => {}
    };

    let (window_width, window_height) = window.get_size();
    let (window_width, window_height): (usize, usize) = (
        window_width.try_into().unwrap(),
        window_height.try_into().unwrap(),
    );

    if window.get_mouse_button(glfw::MouseButtonMiddle) == glfw::Action::Press {
        if window.get_key(glfw::Key::LeftShift) == glfw::Action::Press {
            camera.pan(
                last_cursor.0,
                last_cursor.1,
                cursor.0,
                cursor.1,
                1.0,
                window_width,
                window_height,
            );
        } else if window.get_key(glfw::Key::LeftControl) == glfw::Action::Press {
            camera.move_forward(last_cursor.1, cursor.1, window_height);
        } else {
            camera.rotate_wrt_camera_origin(
                last_cursor.0,
                last_cursor.1,
                cursor.0,
                cursor.1,
                0.1,
                false,
            );
        }
    }

    *last_cursor = cursor;
}