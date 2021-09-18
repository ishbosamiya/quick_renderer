use std::convert::TryInto;

use egui::{FontDefinitions, FontFamily, TextStyle};
use egui_glfw::EguiBackend;
use glfw::{Action, Context, Key};

use quick_renderer::bvh;
use quick_renderer::bvh::BVHDrawData;
use quick_renderer::bvh::BVHTree;
use quick_renderer::bvh::RayHitData;
use quick_renderer::camera::WindowCamera;
use quick_renderer::drawable::Drawable;
use quick_renderer::egui;
use quick_renderer::egui_glfw;
use quick_renderer::fps::FPS;
use quick_renderer::glfw;
use quick_renderer::glm;
use quick_renderer::gpu_immediate::GPUImmediate;
use quick_renderer::gpu_immediate::GPUPrimType;
use quick_renderer::gpu_immediate::GPUVertCompType;
use quick_renderer::gpu_immediate::GPUVertFetchMode;
use quick_renderer::infinite_grid::InfiniteGrid;
use quick_renderer::infinite_grid::InfiniteGridDrawData;
use quick_renderer::mesh;
use quick_renderer::mesh::FaceIndex;
use quick_renderer::mesh::{Mesh, MeshDrawData, MeshUseShader};
use quick_renderer::shader;

// TODO(ish): add bvh axis as a parameter

struct Config {
    bvh: Option<BVHTree<FaceIndex>>,
    draw_bvh: bool,
    bvh_draw_level: usize,
    should_cast_ray: bool,
    bvh_tree_type: u8,
    _bvh_axis: u8,
    bvh_nearest_point_dist: f64,
    bvh_nearest_point_from: glm::DVec3,
    bvh_color: glm::DVec4,
    bvh_ray_color: glm::DVec4,
    bvh_ray_intersection: Vec<(glm::DVec3, RayHitData<FaceIndex>)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bvh: None,
            draw_bvh: true,
            bvh_draw_level: 0,
            should_cast_ray: false,
            bvh_tree_type: 4,
            _bvh_axis: 8,
            bvh_nearest_point_dist: 10.0,
            bvh_nearest_point_from: glm::vec3(2.0, 0.0, 0.0),
            bvh_color: glm::vec4(0.9, 0.5, 0.2, 1.0),
            bvh_ray_color: glm::vec4(0.2, 0.5, 0.9, 1.0),
            bvh_ray_intersection: Vec::new(),
        }
    }
}

impl Config {
    fn build_bvh<END, EVD, EED, EFD>(&mut self, mesh: &Mesh<END, EVD, EED, EFD>, epsilon: f64) {
        let mut bvh = BVHTree::new(mesh.get_faces().len(), epsilon, self.bvh_tree_type, 8);

        mesh.get_faces().iter().for_each(|(_, face)| {
            let co = face
                .get_verts()
                .iter()
                .map(|v_index| {
                    mesh.get_node(mesh.get_vert(*v_index).unwrap().get_node().unwrap())
                        .unwrap()
                        .pos
                })
                .collect();

            bvh.insert(face.get_self_index(), co);
        });

        bvh.balance();

        self.bvh = Some(bvh)
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // set to opengl 3.3 or higher
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    // creating window
    let (mut window, events) = glfw
        .create_window(1280, 720, "BVH Render", glfw::WindowMode::Windowed)
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

    let mut last_cursor = window.get_cursor_pos();

    let mut fps = FPS::default();

    let infinite_grid = InfiniteGrid::default();

    let mut config = Config::default();
    config.build_bvh(mesh, 0.1);

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            handle_window_event(
                &event,
                &mut window,
                &mut camera,
                &mut config,
                &mut last_cursor,
            );
        });

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

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
        }

        unsafe {
            gl::Disable(gl::BLEND);
        }

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("model\0", &glm::identity());
        mesh.draw(&mut MeshDrawData::new(
            &mut imm,
            MeshUseShader::DirectionalLight,
            None,
        ))
        .unwrap();

        {
            draw_sphere_at(
                &config.bvh_nearest_point_from,
                0.02,
                glm::vec4(1.0, 0.2, 0.5, 1.0),
                &mut imm,
            );
        }

        let bvh = config.bvh.as_ref().unwrap();

        bvh.draw(&mut BVHDrawData::new(
            &mut imm,
            config.bvh_draw_level,
            config.bvh_color,
        ))
        .unwrap();

        let op_bvh_nearest_point_data = bvh.find_nearest(
            config.bvh_nearest_point_from,
            config.bvh_nearest_point_dist * config.bvh_nearest_point_dist,
        );

        if let Some(bvh_nearest_point_data) = &op_bvh_nearest_point_data {
            draw_sphere_at(
                &bvh_nearest_point_data.get_co().unwrap(),
                0.02,
                glm::vec4(1.0, 0.2, 0.5, 1.0),
                &mut imm,
            );

            draw_lines(
                &[
                    bvh_nearest_point_data.get_co().unwrap(),
                    config.bvh_nearest_point_from,
                ],
                glm::vec4(1.0, 0.2, 0.5, 1.0),
                &mut imm,
            );
        }

        if config.should_cast_ray {
            let ray_direction = camera.get_raycast_direction(
                last_cursor.0,
                last_cursor.1,
                window_width,
                window_height,
            );

            if let Some(ray_hit_info) = config.bvh.as_ref().unwrap().ray_cast(
                camera.get_position(),
                ray_direction,
                None::<&fn((&glm::DVec3, &glm::DVec3), _) -> Option<bvh::RayHitData<_>>>,
            ) {
                config
                    .bvh_ray_intersection
                    .push((camera.get_position(), ray_hit_info));
            }

            config.should_cast_ray = false;
        }

        {
            if !config.bvh_ray_intersection.is_empty() {
                let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
                    .as_ref()
                    .unwrap();
                smooth_color_3d_shader.use_shader();
                smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

                let format = imm.get_cleared_vertex_format();
                let pos_attr = format.add_attribute(
                    "in_pos\0".to_string(),
                    GPUVertCompType::F32,
                    3,
                    GPUVertFetchMode::Float,
                );
                let color_attr = format.add_attribute(
                    "in_color\0".to_string(),
                    GPUVertCompType::F32,
                    4,
                    GPUVertFetchMode::Float,
                );

                imm.begin(
                    GPUPrimType::Lines,
                    config.bvh_ray_intersection.len() * 2,
                    smooth_color_3d_shader,
                );

                let bvh_ray_color: glm::Vec4 = glm::convert(config.bvh_ray_color);

                config
                    .bvh_ray_intersection
                    .iter()
                    .for_each(|(pos, ray_hit_info)| {
                        let p1: glm::Vec3 = glm::convert(*pos);
                        let p2: glm::Vec3 = glm::convert(ray_hit_info.data.as_ref().unwrap().co);

                        imm.attr_4f(
                            color_attr,
                            bvh_ray_color[0],
                            bvh_ray_color[1],
                            bvh_ray_color[2],
                            bvh_ray_color[3],
                        );
                        imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
                        imm.attr_4f(
                            color_attr,
                            bvh_ray_color[0],
                            bvh_ray_color[1],
                            bvh_ray_color[2],
                            bvh_ray_color[3],
                        );
                        imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
                    });

                imm.end();
            }
        }

        // Keep meshes that have shaders that need alpha channel
        // (blending) bellow this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            if let Some(bvh_nearest_point_data) = &op_bvh_nearest_point_data {
                let radius = glm::distance(
                    &bvh_nearest_point_data.get_co().unwrap(),
                    &config.bvh_nearest_point_from,
                );

                draw_sphere_at(
                    &config.bvh_nearest_point_from,
                    radius,
                    glm::vec4(1.0, 0.2, 0.5, 0.2),
                    &mut imm,
                );
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
                ui.label("BVH Render");
                ui.label(format!("fps: {:.2}", fps.update_and_get(Some(60.0))));

                ui.checkbox(&mut config.draw_bvh, "Draw BVH");
                ui.add(
                    egui::Slider::new(&mut config.bvh_tree_type, 2..=32)
                        .text("BVH Tree Type")
                        .clamp_to_range(true),
                );
                // ui.add(
                //     egui::Slider::new(&mut config.bvh_axis, 6..=26)
                //         .text("BVH Axis")
                //         .clamp_to_range(true),
                // );
                ui.add(
                    egui::Slider::new(&mut config.bvh_nearest_point_dist, 0.0..=40.0)
                        .text("Nearest Point Distance"),
                );
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut config.bvh_nearest_point_from[0], -2.0..=2.0).text("X"),
                );
                ui.add(
                    egui::Slider::new(&mut config.bvh_nearest_point_from[1], -2.0..=2.0).text("Y"),
                );
                ui.add(
                    egui::Slider::new(&mut config.bvh_nearest_point_from[2], -2.0..=2.0).text("Z"),
                );
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut config.bvh_draw_level, 0..=15).text("BVH Draw Level"),
                );
                color_edit_button_dvec4(ui, "BVH Color", &mut config.bvh_color);
                color_edit_button_dvec4(ui, "BVH Ray Color", &mut config.bvh_ray_color);

                if ui.button("Rebuild BVH").clicked() {
                    config.build_bvh(mesh, 0.1);
                }
                if ui.button("Delete Rays").clicked() {
                    config.bvh_ray_intersection.clear();
                }
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
    config: &mut Config,
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

    if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press
        && window.get_key(glfw::Key::LeftControl) == glfw::Action::Press
    {
        config.should_cast_ray = true;
    }

    *last_cursor = cursor;
}

fn color_edit_dvec4(ui: &mut egui::Ui, color: &mut glm::DVec4) {
    let mut color_egui = egui::Color32::from_rgba_premultiplied(
        (color[0] * 255.0) as _,
        (color[1] * 255.0) as _,
        (color[2] * 255.0) as _,
        (color[3] * 255.0) as _,
    );
    egui::color_picker::color_edit_button_srgba(
        ui,
        &mut color_egui,
        egui::color_picker::Alpha::BlendOrAdditive,
    );
    *color = glm::vec4(
        color_egui.r() as f64 / 255.0,
        color_egui.g() as f64 / 255.0,
        color_egui.b() as f64 / 255.0,
        color_egui.a() as f64 / 255.0,
    );
}

fn color_edit_button_dvec4(ui: &mut egui::Ui, text: &str, color: &mut glm::DVec4) {
    ui.horizontal(|ui| {
        ui.label(text);
        color_edit_dvec4(ui, color);
    });
}

fn draw_sphere_at(pos: &glm::DVec3, radius: f64, color: glm::Vec4, imm: &mut GPUImmediate) {
    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();
    smooth_color_3d_shader.use_shader();
    smooth_color_3d_shader.set_mat4(
        "model\0",
        &glm::convert(glm::scale(
            &glm::translate(&glm::identity(), pos),
            &glm::vec3(radius, radius, radius),
        )),
    );

    let ico_sphere = mesh::builtins::get_ico_sphere_subd_01();

    ico_sphere
        .draw(&mut MeshDrawData::new(
            imm,
            mesh::MeshUseShader::SmoothColor3D,
            Some(color),
        ))
        .unwrap();
}

fn draw_lines(positions: &[glm::DVec3], color: glm::Vec4, imm: &mut GPUImmediate) {
    assert_ne!(positions.len(), 0);
    assert!(positions.len() % 2 == 0);
    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();

    smooth_color_3d_shader.use_shader();
    smooth_color_3d_shader.set_mat4("model\0", &glm::identity());

    let format = imm.get_cleared_vertex_format();
    let pos_attr = format.add_attribute(
        "in_pos\0".to_string(),
        GPUVertCompType::F32,
        3,
        GPUVertFetchMode::Float,
    );
    let color_attr = format.add_attribute(
        "in_color\0".to_string(),
        GPUVertCompType::F32,
        4,
        GPUVertFetchMode::Float,
    );

    imm.begin_at_most(
        GPUPrimType::Lines,
        positions.len() * 2,
        smooth_color_3d_shader,
    );

    positions.iter().for_each(|pos| {
        let pos: glm::Vec3 = glm::convert(*pos);
        imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
        imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);
    });

    imm.end();
}
