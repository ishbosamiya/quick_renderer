extern crate image;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use quick_renderer::camera::WindowCamera;
use quick_renderer::drawable::Drawable;
use quick_renderer::egui;
use quick_renderer::egui_glfw;
use quick_renderer::fps::FPS;
use quick_renderer::glfw;
use quick_renderer::glm;
use quick_renderer::gpu_immediate::GPUImmediate;
use quick_renderer::infinite_grid::InfiniteGrid;
use quick_renderer::infinite_grid::InfiniteGridDrawData;
use quick_renderer::mesh;
use quick_renderer::mesh::{MeshDrawData, MeshUseShader};
use quick_renderer::shader;

use std::convert::TryInto;

use gl::types::{GLuint, GLvoid};

pub struct TextureRGBAFloat {
    width: usize,
    height: usize,
    pixels: Vec<(f32, f32, f32, f32)>,

    gl_tex: GLuint,
}

impl TextureRGBAFloat {
    pub fn new_empty(width: usize, height: usize) -> Self {
        let gl_tex = Self::gen_gl_texture();
        let pixels = Vec::new();
        let res = Self {
            width,
            height,
            pixels,
            gl_tex,
        };

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F.try_into().unwrap(),
                res.width.try_into().unwrap(),
                res.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::FLOAT,
                std::ptr::null(),
            )
        }

        res
    }

    pub fn from_image(tex: &image::RgbaImage) -> Self {
        let gl_tex = Self::gen_gl_texture();
        let res = Self {
            width: tex.dimensions().0.try_into().unwrap(),
            height: tex.dimensions().1.try_into().unwrap(),
            pixels: tex
                .pixels()
                .map(|a| {
                    (
                        a.0[0] as f32 / 255.0,
                        a.0[1] as f32 / 255.0,
                        a.0[2] as f32 / 255.0,
                        a.0[3] as f32 / 255.0,
                    )
                })
                .collect(),
            gl_tex,
        };

        assert_eq!(res.pixels.len(), res.width * res.height);

        res.new_texture_to_gl();

        res
    }

    pub fn activate(&mut self, texture_target: u8) {
        let target = match texture_target {
            0 => gl::TEXTURE0,
            1 => gl::TEXTURE1,
            2 => gl::TEXTURE2,
            3 => gl::TEXTURE3,
            4 => gl::TEXTURE4,
            5 => gl::TEXTURE5,
            6 => gl::TEXTURE6,
            7 => gl::TEXTURE7,
            8 => gl::TEXTURE8,
            9 => gl::TEXTURE9,
            10 => gl::TEXTURE10,
            11 => gl::TEXTURE11,
            12 => gl::TEXTURE12,
            13 => gl::TEXTURE13,
            14 => gl::TEXTURE14,
            15 => gl::TEXTURE15,
            16 => gl::TEXTURE16,
            17 => gl::TEXTURE17,
            18 => gl::TEXTURE18,
            19 => gl::TEXTURE19,
            20 => gl::TEXTURE20,
            21 => gl::TEXTURE21,
            22 => gl::TEXTURE22,
            23 => gl::TEXTURE23,
            24 => gl::TEXTURE24,
            25 => gl::TEXTURE25,
            26 => gl::TEXTURE26,
            27 => gl::TEXTURE27,
            28 => gl::TEXTURE28,
            29 => gl::TEXTURE29,
            30 => gl::TEXTURE30,
            31 => gl::TEXTURE31,
            _ => panic!("Texture target not possible, gl support [0, 32)"),
        };
        unsafe {
            gl::ActiveTexture(target);
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);
        }
    }

    fn new_texture_to_gl(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA32F.try_into().unwrap(),
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::FLOAT,
                self.pixels.as_ptr() as *const GLvoid,
            )
        }
    }

    fn gen_gl_texture() -> GLuint {
        let mut gl_tex = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_tex);
        }
        assert_ne!(gl_tex, 0);

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_tex);

            // wrapping method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );

            // filter method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
        }

        gl_tex
    }

    pub fn get_gl_tex(&self) -> GLuint {
        self.gl_tex
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

impl Drop for TextureRGBAFloat {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.gl_tex);
        }
    }
}

pub struct RenderBuffer {
    gl_renderbuffer: GLuint,
}

impl RenderBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut gl_renderbuffer = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut gl_renderbuffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, gl_renderbuffer);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                width.try_into().unwrap(),
                height.try_into().unwrap(),
            );
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        }

        Self { gl_renderbuffer }
    }

    pub fn get_gl_renderbuffer(&self) -> GLuint {
        self.gl_renderbuffer
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.gl_renderbuffer);
        }
    }
}

pub struct FrameBuffer {
    gl_framebuffer: GLuint,
}

impl FrameBuffer {
    pub fn activiate_default() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn new() -> Self {
        let mut gl_framebuffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut gl_framebuffer);
        }

        Self { gl_framebuffer }
    }

    pub fn activate(&self, texture: &TextureRGBAFloat, renderbuffer: &RenderBuffer) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.gl_framebuffer);
        }

        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.get_gl_tex(),
                0,
            );
        }

        // renderbuffer so that depth testing and such can work
        unsafe {
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                renderbuffer.get_gl_renderbuffer(),
            );
        }

        let status;
        unsafe {
            status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        }
        if status != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("error: framebuffer not complete!");
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.gl_framebuffer);
        }
    }
}

fn render_quad(imm: &mut GPUImmediate, shader: &shader::Shader) {
    let plane_vert_positions = vec![
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(-1.0, 1.0, 0.0), glm::vec2(0.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(1.0, -1.0, 0.0), glm::vec2(1.0, 0.0)),
    ];

    let format = imm.get_cleared_vertex_format();
    let pos_attr = format.add_attribute(
        "in_pos\0".to_string(),
        quick_renderer::gpu_immediate::GPUVertCompType::F32,
        3,
        quick_renderer::gpu_immediate::GPUVertFetchMode::Float,
    );
    let uv_attr = format.add_attribute(
        "in_uv\0".to_string(),
        quick_renderer::gpu_immediate::GPUVertCompType::F32,
        2,
        quick_renderer::gpu_immediate::GPUVertFetchMode::Float,
    );

    imm.begin(quick_renderer::gpu_immediate::GPUPrimType::Tris, 6, shader);

    plane_vert_positions.iter().for_each(|(pos, uv)| {
        imm.attr_2f(uv_attr, uv[0], uv[1]);
        imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);
    });

    imm.end();
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // creating window
    let (mut window, events) = glfw
        .create_window(1280, 720, "Simple Render", glfw::WindowMode::Windowed)
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

    let mesh = mesh::builtins::get_ico_sphere_subd_02();

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

    let test_shader =
        shader::Shader::from_strings(include_str!("test.vert"), include_str!("test.frag")).unwrap();

    let jfa_initialization_shader = shader::Shader::from_strings(
        include_str!("jfa_initialization.vert"),
        include_str!("jfa_initialization.frag"),
    )
    .unwrap();

    let jfa_step_shader =
        shader::Shader::from_strings(include_str!("jfa_step.vert"), include_str!("jfa_step.frag"))
            .unwrap();

    let jfa_convert_to_distance_shader = shader::Shader::from_strings(
        include_str!("jfa_convert_to_distance.vert"),
        include_str!("jfa_convert_to_distance.frag"),
    )
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
        "test: uniforms: {:?} attributes: {:?}",
        test_shader.get_uniforms(),
        test_shader.get_attributes(),
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

    let mut loaded_image = TextureRGBAFloat::from_image(&image::imageops::flip_vertical(
        &image::open("test.png").unwrap().into_rgba8(),
    ));

    let framebuffer = FrameBuffer::new();

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum JFA {
        Steps(usize),
        Width(usize),
    }
    let mut jfa = JFA::Steps(0);
    let mut jfa_convert_to_distance = false;

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(&event, &mut window, &mut camera, &mut last_cursor);
        });

        let (width, height) = window.get_size();
        let (width, height): (usize, usize) =
            (width.try_into().unwrap(), height.try_into().unwrap());

        let projection_matrix = &glm::convert(camera.get_projection_matrix(&window));
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
                test_shader.use_shader();
                test_shader.set_mat4("projection\0", projection_matrix);
                test_shader.set_mat4("view\0", view_matrix);
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

        // Jump flood algorithm
        {
            unsafe {
                gl::Disable(gl::DEPTH_TEST);
            }
            let mut jfa_texture_1 = TextureRGBAFloat::new_empty(width, height);
            let mut jfa_texture_2 = TextureRGBAFloat::new_empty(width, height);
            let renderbuffer = RenderBuffer::new(width, height);
            // Initialization
            {
                framebuffer.activate(&jfa_texture_1, &renderbuffer);
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                jfa_initialization_shader.use_shader();
                jfa_initialization_shader.set_int("image\0", 31);
                loaded_image.activate(31);

                render_quad(&mut imm, &jfa_initialization_shader);
            }

            // JFA steps
            {
                match jfa {
                    JFA::Steps(jfa_num_steps) => {
                        (0..jfa_num_steps).for_each(|step| {
                            let render_to;
                            let render_from;
                            if step % 2 == 0 {
                                render_from = &mut jfa_texture_1;
                                render_to = &jfa_texture_2;
                            } else {
                                render_from = &mut jfa_texture_2;
                                render_to = &jfa_texture_1;
                            }

                            framebuffer.activate(render_to, &renderbuffer);
                            unsafe {
                                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                            }

                            let step_size =
                                2.0_f32.powi((jfa_num_steps - 1 - step).try_into().unwrap());

                            jfa_step_shader.use_shader();
                            jfa_step_shader.set_int("image\0", 31);
                            jfa_step_shader.set_float("step_size\0", step_size);
                            render_from.activate(31);

                            render_quad(&mut imm, &jfa_step_shader);
                        });
                    }
                    JFA::Width(width) => {
                        let mut step_size = width as f32;
                        let mut step = 0;
                        while step_size > 0.0 {
                            let render_to;
                            let render_from;
                            if step % 2 == 0 {
                                render_from = &mut jfa_texture_1;
                                render_to = &jfa_texture_2;
                            } else {
                                render_from = &mut jfa_texture_2;
                                render_to = &jfa_texture_1;
                            }

                            framebuffer.activate(render_to, &renderbuffer);
                            unsafe {
                                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
                                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                            }

                            jfa_step_shader.use_shader();
                            jfa_step_shader.set_int("image\0", 31);
                            jfa_step_shader.set_float("step_size\0", step_size);
                            render_from.activate(31);

                            render_quad(&mut imm, &jfa_step_shader);

                            step_size = (step_size / 2.0).floor();
                            step += 1;
                        }
                    }
                }
            }

            unsafe {
                gl::Enable(gl::DEPTH_TEST);
            }

            FrameBuffer::activiate_default();

            // Render out the final texture on a plane in 3D space
            {
                let final_jfa_texture;
                let other_texture;
                match jfa {
                    JFA::Steps(jfa_num_steps) => {
                        if jfa_num_steps % 2 == 0 {
                            final_jfa_texture = &mut jfa_texture_2;
                            other_texture = &mut jfa_texture_1;
                        } else {
                            final_jfa_texture = &mut jfa_texture_1;
                            other_texture = &mut jfa_texture_2;
                        }
                    }
                    JFA::Width(width) => {
                        let width = width as f32;
                        if width.log2().ceil() as i32 % 2 == 0 {
                            final_jfa_texture = &mut jfa_texture_2;
                            other_texture = &mut jfa_texture_1;
                        } else {
                            final_jfa_texture = &mut jfa_texture_1;
                            other_texture = &mut jfa_texture_2;
                        }
                    }
                }
                let final_texture;
                if jfa_convert_to_distance {
                    framebuffer.activate(other_texture, &renderbuffer);

                    jfa_convert_to_distance_shader.use_shader();
                    jfa_convert_to_distance_shader.set_int("image\0", 31);
                    final_jfa_texture.activate(31);

                    render_quad(&mut imm, &jfa_convert_to_distance_shader);

                    final_texture = other_texture;
                    FrameBuffer::activiate_default();
                } else {
                    final_texture = final_jfa_texture;
                }

                flat_texture_shader.use_shader();
                flat_texture_shader.set_int("image\0", 31);
                flat_texture_shader.set_mat4(
                    "model\0",
                    &glm::translate(
                        &glm::scale(
                            &glm::identity(),
                            &glm::vec3(width as f32 / height as f32, 1.0, 1.0),
                        ),
                        &glm::vec3(2.0, 1.0, 0.0),
                    ),
                );
                final_texture.activate(31);
                render_quad(&mut imm, flat_texture_shader);
            }
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
                egui::ComboBox::from_label("JFA Type")
                    .selected_text(format!("{:?}", jfa))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut jfa, JFA::Steps(0), "JFA Steps");
                        ui.selectable_value(&mut jfa, JFA::Width(0), "JFA Width");
                    });
                {
                    match &mut jfa {
                        JFA::Steps(steps) => {
                            ui.add(
                                egui::Slider::new(steps, 0..=30)
                                    .text("JFA Num Steps")
                                    .clamp_to_range(true),
                            );
                        }
                        JFA::Width(width) => {
                            ui.add(egui::Slider::new(width, 0..=75).text("JFA width"));
                        }
                    }
                }
                ui.checkbox(&mut jfa_convert_to_distance, "JFA Convert To Distance");
            });
            let (width, height) = window.get_framebuffer_size();
            let _output = egui.end_frame(glm::vec2(width as _, height as _));
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

    if window.get_mouse_button(glfw::MouseButtonMiddle) == glfw::Action::Press {
        if window.get_key(glfw::Key::LeftShift) == glfw::Action::Press {
            camera.pan(
                last_cursor.0,
                last_cursor.1,
                cursor.0,
                cursor.1,
                1.0,
                window,
            );
        } else if window.get_key(glfw::Key::LeftControl) == glfw::Action::Press {
            camera.move_forward(last_cursor.1, cursor.1, window);
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
