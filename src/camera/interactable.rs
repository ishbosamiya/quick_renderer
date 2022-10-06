use super::Camera;

use std::convert::TryFrom;

/// Interactable camera.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InteractableCamera {
    /// Camera.
    camera: Camera,

    /// Is FPS mode active?
    #[serde(skip, default = "InteractableCamera::default_fps_mode")]
    fps_mode: bool,
    /// Movement speed when FPS mode is active.
    fps_movement_speed: f64,
    /// Rotation speed when FPS mode is active.
    fps_rotation_speed: f64,

    /// Previous frame's cursor position.
    #[serde(skip)]
    last_cursor: Option<(f64, f64)>,
}

impl InteractableCamera {
    /// Default fps_mode.
    const fn default_fps_mode() -> bool {
        false
    }
}

impl InteractableCamera {
    /// Create a new [`InteractableCamera`].
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            fps_mode: Self::default_fps_mode(),
            fps_movement_speed: 5.0,
            fps_rotation_speed: 6.0,
            last_cursor: None,
        }
    }

    /// Interact the camera given the [`glfw::WindowEvent`].
    ///
    /// The last cursor position and the current cursor position must
    /// be provided.
    ///
    /// Returns [`true`] if the [`glfw::WindowEvent`] is consumed.
    ///
    /// # Note
    ///
    /// It is important to call this function every frame (if it is
    /// used) since it needs to update some parameters internally
    /// every frame.
    pub fn interact_glfw_window_event(
        &mut self,
        event: &glfw::WindowEvent,
        window: &glfw::Window,
    ) -> bool {
        let cursor = window.get_cursor_pos();
        let last_cursor = self.last_cursor.unwrap_or(cursor);

        let render_size = window.get_size();
        let render_width = usize::try_from(render_size.0).unwrap();
        let render_height = usize::try_from(render_size.1).unwrap();

        let res = match event {
            glfw::WindowEvent::Key(
                glfw::Key::F,
                _,
                glfw::Action::Press,
                glfw::Modifiers::Control,
            ) if !self.fps_mode => {
                self.fps_mode = true;
                true
            }
            glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, mods)
                if mods.is_empty() && self.fps_mode =>
            {
                self.fps_mode = false;
                true
            }
            glfw::WindowEvent::Scroll(_, scroll_y) => {
                self.camera.zoom(*scroll_y);
                true
            }
            _ => false,
        };

        let res = if !res {
            if self.fps_mode {
                todo!()
            } else {
                let mut pan = false;
                let mut move_foward = false;
                let mut rotate = false;
                if window.get_mouse_button(glfw::MouseButtonMiddle) == glfw::Action::Press
                    || (window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Action::Press
                        && window.get_key(glfw::Key::LeftAlt) == glfw::Action::Press)
                {
                    if window.get_key(glfw::Key::LeftShift) == glfw::Action::Press {
                        pan = true;
                    } else if window.get_key(glfw::Key::LeftControl) == glfw::Action::Press {
                        move_foward = true;
                    } else {
                        rotate = true;
                    }
                }

                if pan {
                    self.camera.pan(
                        last_cursor.0,
                        last_cursor.1,
                        cursor.0,
                        cursor.1,
                        1.0,
                        render_width,
                        render_height,
                    );
                }
                if move_foward {
                    self.camera
                        .move_forward(last_cursor.1, cursor.1, render_height);
                }
                if rotate {
                    self.camera.rotate_wrt_camera_origin(
                        last_cursor.0,
                        last_cursor.1,
                        cursor.0,
                        cursor.1,
                        0.1,
                        false,
                    );
                }

                pan || move_foward || rotate
            }
        } else {
            res
        };

        self.last_cursor = Some(cursor);

        res
    }

    /// Get inner [`Camera`] by reference.
    pub fn get_inner(&self) -> &Camera {
        &self.camera
    }

    /// Get inner [`Camera`] as a mutable reference.
    pub fn get_inner_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Is FPS mode active?
    pub fn get_fps_mode(&self) -> bool {
        self.fps_mode
    }

    /// Set the FPS mode of the camera.
    pub fn set_fps_mode(&mut self, fps_mode: bool) {
        self.fps_mode = fps_mode;
    }

    /// Get the movement speed for when FPS mode is active.
    pub fn get_fps_movement_speed(&self) -> f64 {
        self.fps_movement_speed
    }

    /// Set the FPS movement speed of the camera.
    pub fn set_fps_movement_speed(&mut self, fps_movement_speed: f64) {
        self.fps_movement_speed = fps_movement_speed;
    }

    /// Get the rotation speed for when FPS mode is active.
    pub fn get_fps_rotation_speed(&self) -> f64 {
        self.fps_rotation_speed
    }

    /// Set the FPS rotation speed of the camera.
    pub fn set_fps_rotation_speed(&mut self, fps_rotation_speed: f64) {
        self.fps_rotation_speed = fps_rotation_speed;
    }
}
