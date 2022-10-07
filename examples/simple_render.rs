use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use egui_glfw::EguiBackend;

use quick_renderer::app::App;
use quick_renderer::app::Environment;
use quick_renderer::camera::Camera;
use quick_renderer::camera::InteractableCamera;
use quick_renderer::drawable::Drawable;
use quick_renderer::egui;
use quick_renderer::egui_glfw;
use quick_renderer::glfw;
use quick_renderer::glm;
use quick_renderer::gpu_immediate::GPUImmediate;
use quick_renderer::infinite_grid::InfiniteGrid;
use quick_renderer::infinite_grid::InfiniteGridDrawData;
use quick_renderer::mesh;
use quick_renderer::mesh::simple;
use quick_renderer::mesh::{MeshDrawData, MeshUseShader};
use quick_renderer::shader;

pub struct Application {
    egui: EguiBackend,
    imm: Rc<RefCell<GPUImmediate>>,

    camera: InteractableCamera,
    infinite_grid: InfiniteGrid,

    mesh: &'static simple::Mesh,
}

impl App for Application {
    fn init(environment: &mut Environment) -> Result<Self, Box<dyn std::error::Error>> {
        shader::builtins::display_uniform_and_attribute_info();

        // setup the egui backend
        let egui = EguiBackend::new(&mut environment.window, &mut environment.glfw);

        // larger text
        let mut style = (*egui.get_egui_ctx().style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(18.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        egui.get_egui_ctx().set_style(style);

        Ok(Application {
            egui,
            imm: Rc::new(RefCell::new(GPUImmediate::new())),
            camera: InteractableCamera::new(Camera::new(
                glm::vec3(0.0, 0.0, 3.0),
                glm::vec3(0.0, 1.0, 0.0),
                -90.0,
                0.0,
                45.0,
                None,
            )),
            infinite_grid: InfiniteGrid::default(),
            mesh: mesh::builtins::get_cube_subd_00(),
        })
    }

    fn update(&mut self, environment: &mut Environment) -> Result<(), Box<dyn std::error::Error>> {
        if self.camera.get_fps_mode() {
            environment
                .window
                .set_cursor_mode(glfw::CursorMode::Disabled);
        } else {
            environment.window.set_cursor_mode(glfw::CursorMode::Normal);
        }

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let (window_width, window_height) = environment.window.get_size();
        let (window_width, window_height): (usize, usize) = (
            window_width.try_into().unwrap(),
            window_height.try_into().unwrap(),
        );

        // Shader stuff
        shader::builtins::setup_shaders(self.camera.get_inner(), window_width, window_height);

        unsafe {
            gl::Disable(gl::BLEND);
        }

        let directional_light_shader = shader::builtins::get_directional_light_shader()
            .as_ref()
            .unwrap();

        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();

        let face_orientation_shader = shader::builtins::get_face_orientation_shader()
            .as_ref()
            .unwrap();

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4(
            "model\0",
            &glm::translate(&glm::identity(), &glm::vec3(2.1, 0.0, 0.0)),
        );
        self.mesh
            .draw(&MeshDrawData::new(
                self.imm.clone(),
                MeshUseShader::DirectionalLight,
                None,
            ))
            .unwrap();

        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4(
            "model\0",
            &glm::translate(&glm::identity(), &glm::vec3(-2.1, 0.0, 0.0)),
        );
        self.mesh
            .draw(&MeshDrawData::new(
                self.imm.clone(),
                MeshUseShader::SmoothColor3D,
                Some(glm::vec4(1.0, 0.2, 0.5, 1.0)),
            ))
            .unwrap();

        face_orientation_shader.use_shader();
        face_orientation_shader.set_mat4(
            "model\0",
            &glm::translate(&glm::identity(), &glm::vec3(0.0, 2.1, 0.0)),
        );
        self.mesh
            .draw(&MeshDrawData::new(
                self.imm.clone(),
                MeshUseShader::FaceOrientation,
                None,
            ))
            .unwrap();

        // Keep meshes that have shaders that need alpha channel
        // (blending) bellow this and handle it properly
        {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            self.infinite_grid
                .draw(&InfiniteGridDrawData::new(
                    self.imm.clone(),
                    glm::vec4(0.2, 0.2, 0.2, 1.0),
                ))
                .unwrap();
        }

        // GUI starts
        {
            self.egui
                .begin_frame(&environment.window, &mut environment.glfw);
            egui::Window::new("Hello world!").show(self.egui.get_egui_ctx(), |ui| {
                ui.label("Hello World, Simple Render!");
                ui.label(format!(
                    "fps: {:.2}",
                    environment.fps.update_and_get(Some(60.0))
                ));
            });
            let _output = self
                .egui
                .end_frame(glm::vec2(window_width as _, window_height as _));
        }
        // GUI ends

        Ok(())
    }

    fn handle_window_event(
        &mut self,
        event: &glfw::WindowEvent,
        window: &mut glfw::Window,
        _key_mods: &glfw::Modifiers,
    ) {
        self.egui.handle_event(event, window);

        if let glfw::WindowEvent::FramebufferSize(width, height) = event {
            unsafe {
                gl::Viewport(0, 0, *width, *height);
            }
        };

        self.camera.interact_glfw_window_event(event, window);
    }
}

fn main() {
    Environment::new("Simple Render")
        .unwrap()
        .run::<Application>()
        .unwrap();
}
