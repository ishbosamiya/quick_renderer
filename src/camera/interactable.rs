use egui_glfw::egui;

use super::{Camera, Direction};

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
    /// Previous frame's [`std::time::Instant::now()`].
    #[serde(skip)]
    last_frame_instant: Option<std::time::Instant>,
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
            last_frame_instant: None,
        }
    }

    /// Interact with the camera given the [`glfw::WindowEvent`].
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
        let last_frame_instant = self.last_frame_instant;
        let delta_time = last_frame_instant
            .as_ref()
            .map(|last_frame_instant| last_frame_instant.elapsed().as_secs_f64().min(1.0 / 30.0))
            .unwrap_or(1.0 / 30.0);

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
                self.camera.fps_rotate(
                    cursor.0 - last_cursor.0,
                    last_cursor.1 - cursor.1,
                    self.fps_rotation_speed,
                    delta_time,
                );

                match event {
                    glfw::WindowEvent::Key(
                        glfw::Key::PageUp,
                        _,
                        glfw::Action::Press,
                        glfw::Modifiers::Control | glfw::Modifiers::Shift,
                    ) => {
                        self.fps_movement_speed += 0.3;
                    }
                    glfw::WindowEvent::Key(
                        glfw::Key::PageDown,
                        _,
                        glfw::Action::Press,
                        glfw::Modifiers::Control | glfw::Modifiers::Shift,
                    ) => {
                        self.fps_movement_speed -= 0.3;
                        // clamp the bottom value
                        self.fps_movement_speed = self.fps_movement_speed.max(0.1);
                    }
                    _ => {}
                };

                let movement_speed = match event {
                    glfw::WindowEvent::Key(_, _, _, glfw::Modifiers::Shift) => {
                        // reduce speed
                        Some(self.fps_movement_speed / 2.0)
                    }
                    glfw::WindowEvent::Key(_, _, _, glfw::Modifiers::Control) => {
                        // increase speed
                        Some(self.fps_movement_speed)
                    }
                    glfw::WindowEvent::Key(_, _, _, mods) if mods.is_empty() => {
                        // no change in speed
                        Some(self.fps_movement_speed)
                    }
                    _ => {
                        // no movement
                        None
                    }
                };

                if let Some(movement_speed) = movement_speed {
                    let direction = match event {
                        glfw::WindowEvent::Key(glfw::Key::W, _, _, _) => Some(Direction::Forward),
                        glfw::WindowEvent::Key(glfw::Key::S, _, _, _) => Some(Direction::Backward),
                        glfw::WindowEvent::Key(glfw::Key::A, _, _, _) => Some(Direction::Left),
                        glfw::WindowEvent::Key(glfw::Key::D, _, _, _) => Some(Direction::Right),
                        _ => None,
                    };

                    if let Some(direction) = direction {
                        self.camera.fps_move(direction, movement_speed, delta_time);
                    }
                }

                true
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
        self.last_frame_instant = Some(std::time::Instant::now());

        res
    }

    /// Interact with the camera for events from [`egui`]. It requires
    /// the [`egui::Ui`], the [`egui::Response`] of the UI element
    /// that is interacted with and the dimensions of the surface for
    /// which the camera is used.
    ///
    /// Returns [`true`] if an interaction with the camera took place.
    ///
    /// # Note
    ///
    /// It is important to call this function every frame (if it is
    /// used) since it needs to update some parameters internally
    /// every frame.
    pub fn interact_egui(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        render_width: usize,
        render_height: usize,
    ) -> bool {
        let cursor = if let Some(hover_pos) = ui.input().pointer.hover_pos() {
            (hover_pos.x as f64, hover_pos.y as f64)
        } else {
            return false;
        };
        let last_cursor = self.last_cursor.unwrap_or(cursor);
        let last_frame_instant = self.last_frame_instant;
        let delta_time = last_frame_instant
            .as_ref()
            .map(|last_frame_instant| last_frame_instant.elapsed().as_secs_f64().min(1.0 / 30.0))
            .unwrap_or(1.0 / 30.0);

        if response.hovered()
            && !self.fps_mode
            && ui.input().key_pressed(egui::Key::F)
            && ui.input().modifiers.command_only()
        {
            self.fps_mode = true;
        }

        if self.fps_mode
            && ui.input().key_pressed(egui::Key::Escape)
            && ui.input().modifiers.is_none()
        {
            self.fps_mode = false;
        }

        let fov_changed = if ui.input().scroll_delta.y != 0.0 {
            self.camera.zoom((ui.input().scroll_delta.y as f64) * 0.01);
            true
        } else {
            false
        };

        let res = if self.fps_mode {
            self.camera.fps_rotate(
                cursor.0 - last_cursor.0,
                last_cursor.1 - cursor.1,
                self.fps_rotation_speed,
                delta_time,
            );

            if ui.input().key_down(egui::Key::PageUp)
                && ui
                    .input()
                    .modifiers
                    .matches(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT)
            {
                self.fps_movement_speed += 0.3;
            } else if ui.input().key_down(egui::Key::PageDown)
                && ui
                    .input()
                    .modifiers
                    .matches(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT)
            {
                self.fps_movement_speed -= 0.1;
                // clamp the bottom value
                self.fps_movement_speed = self.fps_movement_speed.max(0.1);
            }

            let movement_speed = if ui.input().modifiers.is_none() {
                // no change
                Some(self.fps_movement_speed)
            } else if ui.input().modifiers.shift_only() {
                // reduce speed
                Some(self.fps_movement_speed / 2.0)
            } else if ui.input().modifiers.command_only() {
                // increase speed
                Some(self.fps_movement_speed * 2.0)
            } else {
                // no movement
                None
            };

            if let Some(movement_speed) = movement_speed {
                if ui.input().key_down(egui::Key::W) {
                    self.camera
                        .fps_move(Direction::Forward, movement_speed, delta_time);
                }
                if ui.input().key_down(egui::Key::S) {
                    self.camera
                        .fps_move(Direction::Backward, movement_speed, delta_time);
                }
                if ui.input().key_down(egui::Key::A) {
                    self.camera
                        .fps_move(Direction::Left, movement_speed, delta_time);
                }
                if ui.input().key_down(egui::Key::D) {
                    self.camera
                        .fps_move(Direction::Right, movement_speed, delta_time);
                }
            }

            true
        } else {
            let mut pan = false;
            let mut move_foward = false;
            let mut rotate = false;
            if response.dragged_by(egui::PointerButton::Middle) {
                if ui.input().modifiers.shift_only() {
                    pan = true;
                } else if ui.input().modifiers.command_only() {
                    move_foward = true;
                } else {
                    rotate = true;
                }
            } else if response.dragged_by(egui::PointerButton::Primary) {
                if ui
                    .input()
                    .modifiers
                    .matches(egui::Modifiers::ALT | egui::Modifiers::SHIFT)
                {
                    pan = true;
                } else if ui
                    .input()
                    .modifiers
                    .matches(egui::Modifiers::ALT | egui::Modifiers::CTRL)
                {
                    move_foward = true;
                } else if ui.input().modifiers.matches(egui::Modifiers::ALT) {
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
        };

        self.last_cursor = Some(cursor);
        self.last_frame_instant = Some(std::time::Instant::now());

        res || fov_changed
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
