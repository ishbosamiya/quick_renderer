use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use egui_glfw::EguiBackend;

use quick_renderer::app::App;
use quick_renderer::app::Environment;
use quick_renderer::app::EnvironmentSettings;
use quick_renderer::app::MaybeContinue;
use quick_renderer::camera::Camera;
use quick_renderer::camera::InteractableCamera;
use quick_renderer::drawable::Drawable;
use quick_renderer::egui;
use quick_renderer::egui_glfw;
use quick_renderer::framebuffer::FrameBuffer;
use quick_renderer::glfw;
use quick_renderer::glm;
use quick_renderer::gpu_immediate::GPUImmediate;
use quick_renderer::infinite_grid::InfiniteGrid;
use quick_renderer::infinite_grid::InfiniteGridDrawData;
use quick_renderer::mesh;
use quick_renderer::mesh::simple;
use quick_renderer::mesh::{MeshDrawData, MeshUseShader};
use quick_renderer::renderbuffer::RenderBuffer;
use quick_renderer::shader;
use quick_renderer::texture::TextureRGBAFloat;

pub struct Application {
    egui: EguiBackend,
    imm: Rc<RefCell<GPUImmediate>>,

    camera: InteractableCamera,
    infinite_grid: InfiniteGrid,

    mesh: &'static simple::Mesh,

    render_texture: TextureRGBAFloat,
}

impl App for Application {
    type InitData = ();

    fn init(
        environment: &mut Environment,
        _extra: Self::InitData,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
            render_texture: TextureRGBAFloat::new_empty(0, 0),
        })
    }

    type ExitData = ();

    fn update(
        &mut self,
        environment: &mut Environment,
    ) -> Result<MaybeContinue<Self::ExitData>, Box<dyn std::error::Error>> {
        if self.camera.get_fps_mode() {
            environment
                .window
                .set_cursor_mode(glfw::CursorMode::Disabled);
        } else {
            environment.window.set_cursor_mode(glfw::CursorMode::Normal);
        }

        self.egui
            .begin_frame(&environment.window, &mut environment.glfw);

        egui::CentralPanel::default().show(&self.egui.get_egui_ctx().clone(), |ui| {
            let render_width = ui.available_width().floor() as usize;
            let render_height = ui.available_height().floor() as usize;

            Self::render_scene(
                &mut self.render_texture,
                self.mesh,
                &self.camera,
                render_width,
                render_height,
                &self.infinite_grid,
                self.imm.clone(),
            );

            let response = ui
                .image(
                    egui::TextureId::User(self.render_texture.get_gl_tex().into()),
                    egui::vec2(render_width as _, render_height as _),
                )
                .interact(egui::Sense::click_and_drag());

            self.camera
                .interact_egui(ui, &response, render_width, render_height);
        });

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let (window_width, window_height) = environment.window.get_size();
        let _output = self.egui.end_frame((window_width as _, window_height as _));

        Ok(MaybeContinue::Continue)
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
    }
}

impl Application {
    fn render_scene(
        render_texture: &mut TextureRGBAFloat,
        mesh: &simple::Mesh,
        camera: &InteractableCamera,
        render_width: usize,
        render_height: usize,
        infinite_grid: &InfiniteGrid,
        imm: Rc<RefCell<GPUImmediate>>,
    ) {
        if render_width != render_texture.get_width()
            || render_height != render_texture.get_height()
        {
            *render_texture = TextureRGBAFloat::new_empty(render_width, render_height);
        }

        let mut prev_viewport_params = [0, 0, 0, 0];
        let prev_depth_enable = unsafe { gl::IsEnabled(gl::DEPTH_TEST) } != 0;
        let prev_blend_enable = unsafe { gl::IsEnabled(gl::BLEND) } != 0;
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, prev_viewport_params.as_mut_ptr());
            gl::Viewport(
                0,
                0,
                render_width.try_into().unwrap(),
                render_height.try_into().unwrap(),
            );
            gl::Enable(gl::DEPTH_TEST);
        }

        let render_buffer = RenderBuffer::new(render_width, render_height);
        let frame_buffer = FrameBuffer::new();
        frame_buffer.activate(render_texture, &render_buffer);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Shader stuff
        shader::builtins::setup_shaders(camera.get_inner(), render_width, render_height);

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
        mesh.draw(&MeshDrawData::new(
            imm.clone(),
            MeshUseShader::DirectionalLight,
            None,
        ))
        .unwrap();

        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4(
            "model\0",
            &glm::translate(&glm::identity(), &glm::vec3(-2.1, 0.0, 0.0)),
        );
        mesh.draw(&MeshDrawData::new(
            imm.clone(),
            MeshUseShader::SmoothColor3D,
            Some(glm::vec4(1.0, 0.2, 0.5, 1.0)),
        ))
        .unwrap();

        face_orientation_shader.use_shader();
        face_orientation_shader.set_mat4(
            "model\0",
            &glm::translate(&glm::identity(), &glm::vec3(0.0, 2.1, 0.0)),
        );
        mesh.draw(&MeshDrawData::new(
            imm.clone(),
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

            infinite_grid
                .draw(&InfiniteGridDrawData::new(
                    imm,
                    glm::vec4(0.2, 0.2, 0.2, 1.0),
                ))
                .unwrap();
        }

        unsafe {
            gl::Viewport(
                prev_viewport_params[0],
                prev_viewport_params[1],
                prev_viewport_params[2],
                prev_viewport_params[3],
            );
            if !prev_depth_enable {
                gl::Disable(gl::DEPTH_TEST);
            }
            if !prev_blend_enable {
                gl::Disable(gl::BLEND);
            }
        }

        FrameBuffer::activiate_default();
    }
}

fn main() {
    Environment::new(
        "Simple Render egui interaction",
        &EnvironmentSettings::default(),
    )
    .unwrap()
    .run::<Application>(())
    .unwrap();
}
