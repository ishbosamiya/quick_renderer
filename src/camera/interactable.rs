use super::Camera;

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
        }
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
