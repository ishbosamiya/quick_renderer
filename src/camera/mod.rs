pub mod interactable;

pub use interactable::InteractableCamera;

use std::{cell::RefCell, fmt::Display, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    drawable::{Drawable, NoSpecificDrawError},
    glm,
    gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode},
    gpu_utils, shader,
    texture::TextureRGBAFloat,
    util,
};

/// Camera.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// position of the camera in 3D space
    position: glm::DVec3,
    /// front direction of the camera
    front: glm::DVec3,
    /// up direction of the camera
    up: glm::DVec3,
    /// right direction of the camera
    right: glm::DVec3,
    /// up direction of the world with respect to which the camera's
    /// front, up and right are defined
    world_up: glm::DVec3,
    /// yaw of the camera
    yaw: f64,
    /// pitch of the camera
    pitch: f64,
    /// vertical field of view of the camera in degrees
    fov: f64,

    /// near clipping plane of the camera
    near_plane: f64,
    /// far clipping plane of the camera
    far_plane: f64,

    /// camera's sensor
    sensor: Option<Sensor>,
}

impl Camera {
    /// Create new camera
    ///
    /// `sensor` is generally set to [`None`] for most camera
    /// instances.
    pub fn new(
        position: glm::DVec3,
        up: glm::DVec3,
        yaw: f64,
        pitch: f64,
        fov: f64,
        sensor: Option<Sensor>,
    ) -> Self {
        let mut camera = Self {
            position,
            yaw,
            pitch,
            world_up: up,
            front: glm::vec3(0.0, 0.0, -1.0),
            right: glm::zero(),
            up,
            fov,
            near_plane: 0.1,
            far_plane: 1000.0,
            sensor,
        };

        camera.update_camera_vectors();

        camera
    }

    /// Calculate the `front`, `right` and `up` vectors of the camera
    /// from the `yaw`, `pitch` and `world_up` values of the camera
    pub fn update_camera_vectors(&mut self) {
        let yaw_radians = f64::to_radians(self.yaw);
        let pitch_radians = f64::to_radians(self.pitch);
        let front: glm::DVec3 = glm::vec3(
            yaw_radians.cos() * pitch_radians.cos(),
            pitch_radians.sin(),
            yaw_radians.sin() * pitch_radians.cos(),
        );
        self.front = glm::normalize(&front);

        self.right = glm::normalize(&glm::cross(&front, &self.world_up));
        self.up = glm::normalize(&glm::cross(&self.right, &front));
    }

    /// Get world up.
    pub fn get_world_up(&self) -> &glm::DVec3 {
        &self.world_up
    }

    /// Get camera position.
    pub fn get_position(&self) -> glm::DVec3 {
        self.position
    }

    /// Get camera front vector.
    pub fn get_front(&self) -> glm::DVec3 {
        self.front
    }

    /// Get camera right vector.
    pub fn get_right(&self) -> glm::DVec3 {
        self.right
    }

    /// Get camera up vector.
    pub fn get_up(&self) -> glm::DVec3 {
        self.up
    }

    /// Get camera near plane distance.
    pub fn get_near_plane(&self) -> f64 {
        self.near_plane
    }

    /// Get camera far plane distance.
    pub fn get_far_plane(&self) -> f64 {
        self.far_plane
    }

    /// Get camera yaw.
    pub fn get_yaw(&self) -> f64 {
        self.yaw
    }

    /// Get camera pitch.
    pub fn get_pitch(&self) -> f64 {
        self.pitch
    }

    /// Get camera field of view (fov) (in degrees).
    ///
    /// Note: this is the vertical fov of the camera. To get the
    /// horizontal fov, convert using [`vfov_to_hfov()`].
    ///
    /// # Example
    ///
    /// ```
    /// # let camera = Camera::new(
    /// #     glm::vec3(0.0, 0.0, 0.0),
    /// #     glm::vec3(0.0, 1.0, 0.0),
    /// #     -90.0,
    /// #     0.0,
    /// #     45.0,
    /// #     None,
    /// # );
    /// let vfov_deg = camera.get_fov();
    /// let vfov_rad = vfov_deg.to_radians();
    /// let hfov_rad = vfov_to_hfov(vfov, 1.0);
    /// assert!((vfov_rad - hfov_rad).abs() < 1e-5);
    /// ```
    pub fn get_fov(&self) -> f64 {
        self.fov
    }

    /// Get the camera focal length.
    pub fn get_focal_length(&self) -> Option<f64> {
        Some(util::fov_to_focal_length(
            self.get_fov().to_radians(),
            self.get_sensor()?.get_height(),
        ))
    }

    /// Get reference to camera's sensor
    pub fn get_sensor(&self) -> Option<&Sensor> {
        self.sensor.as_ref()
    }

    /// Get camera's sensor
    pub fn get_sensor_no_ref(&self) -> Option<Sensor> {
        self.sensor
    }

    /// Get mutable reference to camera's sensor
    pub fn get_sensor_mut(&mut self) -> &mut Option<Sensor> {
        &mut self.sensor
    }

    /// Get the view matrix.
    pub fn get_view_matrix(&self) -> glm::DMat4 {
        glm::look_at(&self.position, &(self.position + self.front), &self.up)
    }

    /// Get the perspective projection matrix.
    #[deprecated(
        since = "0.5.0+dev",
        note = "It is recommended to use get_perspective_projection_matrix() instead."
    )]
    pub fn get_projection_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        self.get_perspective_projection_matrix(width, height)
    }

    /// Get the perspective projection matrix.
    pub fn get_perspective_projection_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        glm::perspective(
            width as f64 / height as f64,
            self.fov.to_radians(),
            self.near_plane,
            self.far_plane,
        )
    }

    /// Get the orthogonal projection matrix.
    pub fn get_ortho_matrix(&self, width: usize, height: usize) -> glm::DMat4 {
        glm::ortho(
            0.0,
            width as f64,
            0.0,
            height as f64,
            self.near_plane,
            self.far_plane,
        )
    }

    /// Pan the camera.
    #[allow(clippy::too_many_arguments)]
    pub fn pan(
        &mut self,
        mouse_start_x: f64,
        mouse_start_y: f64,
        mouse_end_x: f64,
        mouse_end_y: f64,
        len: f64,
        width: usize,
        height: usize,
    ) {
        if (mouse_start_x - mouse_end_x).abs() < f64::EPSILON
            && (mouse_start_y - mouse_end_y).abs() < f64::EPSILON
        {
            return;
        }
        let clip_x = mouse_start_x * 2.0 / width as f64 - 1.0;
        let clip_y = 1.0 - mouse_start_y * 2.0 / height as f64;

        let clip_end_x = mouse_end_x * 2.0 / width as f64 - 1.0;
        let clip_end_y = 1.0 - mouse_end_y * 2.0 / height as f64;

        let inverse_mvp = glm::inverse(
            &(self.get_perspective_projection_matrix(width, height) * self.get_view_matrix()),
        );
        let out_vector = inverse_mvp * glm::vec4(clip_x, clip_y, 0.0, 1.0);
        let world_pos = glm::vec3(
            out_vector.x / out_vector.w,
            out_vector.y / out_vector.w,
            out_vector.z / out_vector.w,
        );

        let out_end_vector = inverse_mvp * glm::vec4(clip_end_x, clip_end_y, 0.0, 1.0);
        let world_pos_2 = glm::vec3(
            out_end_vector.x / out_end_vector.w,
            out_end_vector.y / out_end_vector.w,
            out_end_vector.z / out_end_vector.w,
        );

        let dir = world_pos_2 - world_pos;

        let offset = glm::length(&dir) * glm::normalize(&dir) * self.fov * len / 2.0;

        self.position -= offset;
    }

    /// Rotate the camera with respect to the camera origin (camera
    /// position).
    pub fn rotate_wrt_camera_origin(
        &mut self,
        mouse_start_x: f64,
        mouse_start_y: f64,
        mouse_end_x: f64,
        mouse_end_y: f64,
        mouse_sensitivity: f64,
        constrain_pitch: bool,
    ) {
        let x_offset = (mouse_end_x - mouse_start_x) * mouse_sensitivity;
        let y_offset = (mouse_start_y - mouse_end_y) * mouse_sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        if constrain_pitch {
            if self.pitch > 89.0 {
                self.pitch = 89.0;
            } else if self.pitch < -89.0 {
                self.pitch = -89.0;
            }
        }

        self.update_camera_vectors();
    }

    /// Move the camera forward.
    pub fn move_forward(&mut self, mouse_start_y: f64, mouse_end_y: f64, height: usize) {
        let clip_y = 1.0 - mouse_start_y * 2.0 / height as f64;
        let clip_end_y = 1.0 - mouse_end_y * 2.0 / height as f64;

        let move_by = clip_end_y - clip_y;

        self.position += self.front * move_by;
    }

    /// Zoom the camera. This changes the field of view of the camera.
    pub fn zoom(&mut self, scroll_y: f64) {
        let min = 1.0;
        let max = 90.0;
        if self.fov >= min && self.fov <= max {
            self.fov -= scroll_y;
        }
        if self.fov < min {
            self.fov = min;
        }
        if self.fov > max {
            self.fov = max;
        }
    }

    /// Get the direction of the ray if cast from the camera position
    /// towards the point on the camera plane that is determined by
    /// the given x, y coordinates.
    pub fn get_raycast_direction(
        &mut self,
        mouse_x: f64,
        mouse_y: f64,
        width: usize,
        height: usize,
    ) -> glm::DVec3 {
        let x = (2.0 * mouse_x) / width as f64 - 1.0;
        let y = 1.0 - (2.0 * mouse_y) / height as f64;

        let ray_clip = glm::vec4(x, y, -1.0, 1.0);

        let ray_eye =
            glm::inverse(&self.get_perspective_projection_matrix(width, height)) * ray_clip;
        let ray_eye = glm::vec4(ray_eye[0], ray_eye[1], -1.0, 0.0);

        let ray_wor = glm::inverse(&self.get_view_matrix()) * ray_eye;

        glm::normalize(&glm::vec4_to_vec3(&ray_wor))
    }

    /// Get ray cast direction given the UVs on the camera sensor
    /// through which the ray should pass.
    ///
    /// If no sensor is available in the camera, return None.
    ///
    /// UVs are defined as (0.0, 0.0) at center, (1.0, 1.0) as top
    /// right corner and (-1.0, -1.0) as bottom left corner of the
    /// sensor.
    pub fn get_raycast_direction_uv(&self, uv: &glm::DVec2) -> Option<glm::DVec3> {
        let sensor = self.get_sensor()?;

        let camera_plane_center = self.position
            + self.front
                * self
                    .get_focal_length()
                    .expect("by this point focal length should always be available");

        let horizontal = self.right * sensor.get_width() / 2.0;
        let vertical = self.up * sensor.get_height() / 2.0;

        let point_on_sensor = camera_plane_center + uv[0] * horizontal + uv[1] * vertical;

        Some((point_on_sensor - self.position).normalize())
    }

    /// Set the camera's position.
    pub fn set_position(&mut self, position: glm::DVec3) {
        self.position = position;
    }

    /// Set the camera's focal length
    ///
    /// # Panics
    ///
    /// Panics if camera sensor is not set.
    pub fn set_focal_length(&mut self, focal_length: f64) {
        self.fov = util::focal_length_to_fov(focal_length, self.get_sensor().unwrap().get_height())
            .to_degrees();
    }

    /// Set the yaw of the camera.
    ///
    /// If yaw and pitch of the camera must be set, it is recommended
    /// to use [`Self::set_yaw_and_pitch()`].
    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;
        self.update_camera_vectors();
    }

    /// Set the pitch of the camera.
    ///
    /// If yaw and pitch of the camera must be set, it is recommended
    /// to use [`Self::set_yaw_and_pitch()`].
    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch;
        self.update_camera_vectors();
    }

    /// Set the yaw and pitch of the camera.
    pub fn set_yaw_and_pitch(&mut self, yaw: f64, pitch: f64) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.update_camera_vectors();
    }

    /// Set the camera's near plane.
    pub fn set_near_plane(&mut self, near_plane: f64) {
        self.near_plane = near_plane;
    }

    /// Set the camera's far plane.
    pub fn set_far_plane(&mut self, far_plane: f64) {
        self.far_plane = far_plane;
    }

    /// Move the camera in the given direction.
    ///
    /// `movement_speed`: Speed with which to move the camera.
    ///
    /// `delta_time`: Time between frames to be able to process
    /// the `movement_speed` correctly. This matters when the FPS
    /// changes drastically between frames, if not present, it
    /// lead to moving the camera a different amount each frame
    /// despite the same requested speed.
    pub fn fps_move(&mut self, direction: Direction, movement_speed: f64, delta_time: f64) {
        let distance = movement_speed * delta_time;
        match direction {
            Direction::Forward => {
                self.set_position(self.get_position() + self.get_front() * distance)
            }
            Direction::Backward => {
                self.set_position(self.get_position() - self.get_front() * distance)
            }
            Direction::Left => self.set_position(self.get_position() - self.get_right() * distance),
            Direction::Right => {
                self.set_position(self.get_position() + self.get_right() * distance)
            }
        }
    }

    /// Rotate the camera in a FPS like manner.
    ///
    /// `offset_x`: Distance in the camera plane (units to be
    /// determined but most likely the viewport pixels) moved in
    /// the x direction. This is often the distance in pixels
    /// moved by the cursor between the previous frame and the
    /// current frame.
    ///
    /// `offset_y`: Distance in the camera plane (units to be
    /// determined but most likely the viewport pixels) moved in
    /// the y direction. This is often the distance in pixels
    /// moved by the cursor between the previous frame and the
    /// current frame.
    ///
    /// `rotation_speed`: Speed with which the camera should
    /// rotate.
    ///
    /// `delta_time`: Time between frames to be able to process
    /// the `rotation_speed` correctly. This matters when the FPS
    /// changes drastically between frames, if not present, it
    /// will lead to the camera rotating a different amount each
    /// frame despite the same requested speed.
    pub fn fps_rotate(
        &mut self,
        offset_x: f64,
        offset_y: f64,
        rotation_speed: f64,
        delta_time: f64,
    ) {
        let offset_x = offset_x * rotation_speed * delta_time;
        let offset_y = offset_y * rotation_speed * delta_time;

        self.set_yaw_and_pitch(self.get_yaw() + offset_x, self.get_pitch() + offset_y);
    }

    /// Move the camera to fit the given verts in the camera view.
    ///
    /// `camera_width`: Width of the surface that uses this camera to
    /// render the scene.
    ///
    /// `camera_height`: Height of the surface that uses this camera
    /// to render the scene.
    ///
    /// `verts`: Vertices that must fit into the camera.
    ///
    /// `margin`: Optional margin to add. It is recommended that the
    /// margin provided be contained in `0.0..=1.0`. A good default to
    /// use is `Some(0.3)`.
    pub fn move_to_fit_verts_in_camera_view(
        &mut self,
        camera_width: usize,
        camera_height: usize,
        verts: &[glm::Vec3],
        margin: Option<f32>,
    ) -> Result<(), FitVertsInCameraViewError> {
        if verts.is_empty() {
            return Err(FitVertsInCameraViewError::NoVertsProvided);
        }

        let mut previous_position = self.get_position();
        const MAX_ITERATIONS: usize = 20;
        for _ in 0..MAX_ITERATIONS {
            self.move_to_fit_verts_in_camera_view_impl(camera_width, camera_height, verts, margin)?;
            let position = self.get_position();
            if (position[0] - previous_position[0]).abs() < 1e-5
                && (position[1] - previous_position[1]).abs() < 1e-5
                && (position[2] - previous_position[2]).abs() < 1e-5
            {
                return Ok(());
            }
            previous_position = position;
        }

        println!("warning: move to fit verts in camera view didn't terminate successfully");

        Ok(())
    }

    /// Implementation of a single iteration of
    /// [`Self::move_to_fit_verts_in_camera_view()`].
    fn move_to_fit_verts_in_camera_view_impl(
        &mut self,
        camera_width: usize,
        camera_height: usize,
        verts: &[glm::Vec3],
        margin: Option<f32>,
    ) -> Result<(), FitVertsInCameraViewError> {
        let margin = margin.unwrap_or(0.0);
        let view = &glm::convert::<_, glm::Mat4>(self.get_view_matrix());
        let proj = &glm::convert::<_, glm::Mat4>(
            self.get_perspective_projection_matrix(camera_width, camera_height),
        );

        let calculate_clip_space_bounds_and_view_space_pos_max_z =
            |view: &glm::Mat4, proj: &glm::Mat4| {
                verts
                    .iter()
                    .map(|pos| {
                        let view_space_pos = &Self::apply_matrix_vec3_to_vec4(pos, view);
                        let proj_space_pos = &(proj * view_space_pos);
                        (proj_space_pos / proj_space_pos[3], *view_space_pos)
                    })
                    .fold(
                        (
                            glm::vec2(f32::MAX, f32::MAX),
                            glm::vec2(f32::MIN, f32::MIN),
                            glm::vec4(f32::MIN, f32::MIN, f32::MIN, f32::MIN),
                        ),
                        |(clip_space_min, clip_space_max, view_space_pos_max_z),
                         (clip_space_pos, view_space_pos)| {
                            (
                                glm::min2(&clip_space_min, &glm::vec4_to_vec2(&clip_space_pos)),
                                glm::max2(&clip_space_max, &glm::vec4_to_vec2(&clip_space_pos)),
                                if view_space_pos_max_z[2] > view_space_pos[2] {
                                    view_space_pos_max_z
                                } else {
                                    view_space_pos
                                },
                            )
                        },
                    )
            };

        let (clip_space_min_bounds, clip_space_max_bounds, view_space_pos_max_z) =
            &calculate_clip_space_bounds_and_view_space_pos_max_z(view, proj);

        let (view, proj, clip_space_min_bounds, clip_space_max_bounds, view_space_pos_max_z) =
            &if view_space_pos_max_z[2] >= 0.0 {
                println!("info: moving camera since mesh center is behind the camera");

                // move the camera so that the mesh is just visible, this
                // helps prevent scenarios of no intersection (when the camera
                // is in front of the mesh center plane)

                let inv_view = &glm::inverse(view);

                let new_camera_position = {
                    // offset the new camera position a smidge along the
                    // camera front axis
                    let view_space_new_camera_position = glm::vec4(
                        0.0,
                        0.0,
                        view_space_pos_max_z[2] + 0.1,
                        view_space_pos_max_z[3],
                    );
                    let world_space_new_camera_position =
                        &(inv_view * view_space_new_camera_position);
                    glm::convert(glm::vec4_to_vec3(world_space_new_camera_position))
                };

                self.set_position(new_camera_position);

                let view = &glm::convert::<_, glm::Mat4>(self.get_view_matrix());
                let proj = &glm::convert::<_, glm::Mat4>(
                    self.get_perspective_projection_matrix(camera_width, camera_height),
                );

                let (clip_space_min_bounds, clip_space_max_bounds, view_space_pos_max_z) =
                    calculate_clip_space_bounds_and_view_space_pos_max_z(view, proj);
                (
                    *view,
                    *proj,
                    clip_space_min_bounds,
                    clip_space_max_bounds,
                    view_space_pos_max_z,
                )
            } else {
                (
                    *view,
                    *proj,
                    *clip_space_min_bounds,
                    *clip_space_max_bounds,
                    *view_space_pos_max_z,
                )
            };

        let inv_view = &glm::inverse(view);
        let inv_proj = &glm::inverse(proj);

        let get_view_space_ray_direction = |clip_space_vec: &glm::Vec2| {
            // take the near plane of the camera
            let view_space_eye =
                &(inv_proj * glm::vec4(clip_space_vec[0], clip_space_vec[1], -1.0, 1.0));
            glm::normalize(&glm::vec3(view_space_eye[0], view_space_eye[1], -1.0))
        };

        let expand_bounds_by_margin = |min_bounds: &glm::Vec2, max_bounds: &glm::Vec2| {
            let margin = margin * 0.5;
            let margin = glm::vec2(margin, margin);
            (min_bounds - margin, max_bounds + margin)
        };

        let (clip_space_min_bounds, clip_space_max_bounds) =
            &expand_bounds_by_margin(clip_space_min_bounds, clip_space_max_bounds);

        let view_space_ray_dir_bottom_left = &get_view_space_ray_direction(clip_space_min_bounds);
        let view_space_ray_dir_top_right = &get_view_space_ray_direction(clip_space_max_bounds);

        // the ray origin in the view space is at (0.0, 0.0, 0.0)
        // since the camera position in the view space is at (0.0,
        // 0.0, 0.0)
        let view_space_ray_origin = &glm::vec3(0.0, 0.0, 0.0);
        // the plane normal in the view space is directed towards the
        // viewer, which is in the +z direction in this case
        let view_space_plane_normal = &glm::vec3(0.0, 0.0, 1.0);
        let view_space_point_on_plane = &glm::vec4_to_vec3(view_space_pos_max_z);

        let view_space_t_bottom_left = Self::ray_plane_intersection(
            view_space_ray_origin,
            view_space_ray_dir_bottom_left,
            view_space_point_on_plane,
            view_space_plane_normal,
        );

        let view_space_t_top_right = Self::ray_plane_intersection(
            view_space_ray_origin,
            view_space_ray_dir_top_right,
            view_space_point_on_plane,
            view_space_plane_normal,
        );

        if let Some((bl, tr)) =
            view_space_t_bottom_left.and_then(|bl| view_space_t_top_right.map(|tr| (bl, tr)))
        {
            let get_point = |t: f32, direction: &glm::Vec3| view_space_ray_origin + t * direction;

            let view_space_intersection_bottom_left =
                &get_point(bl, view_space_ray_dir_bottom_left);
            let view_space_intersection_top_right = &get_point(tr, view_space_ray_dir_top_right);

            let expected_viewport_size = &(glm::vec3_to_vec2(view_space_intersection_top_right)
                - glm::vec3_to_vec2(view_space_intersection_bottom_left));

            let camera_aspect = camera_width as f64 / camera_height as f64;
            let vfov = self.get_fov().to_radians();
            let hfov = vfov_to_hfov(vfov, camera_aspect) as f32;
            let vfov = vfov as f32;

            let calculate_expected_distance =
                |fov: f32, size: f32| (size * 0.5) / (fov * 0.5).tan();

            let expected_distance_hfov_x =
                calculate_expected_distance(hfov, expected_viewport_size[0]);
            let expected_distance_vfov_y =
                calculate_expected_distance(vfov, expected_viewport_size[1]);

            let expected_distance = expected_distance_hfov_x.max(expected_distance_vfov_y);

            let view_space_intersection_center =
                &((view_space_intersection_bottom_left + view_space_intersection_top_right) * 0.5);

            let view_space_new_camera_position =
                &(view_space_intersection_center + glm::vec3(0.0, 0.0, expected_distance));

            let view_to_world_space_new_camera_position = &glm::vec4_to_vec3(
                &Self::apply_matrix_vec3_to_vec4(view_space_new_camera_position, inv_view),
            );

            self.set_position(glm::convert(*view_to_world_space_new_camera_position));

            Ok(())
        } else {
            Err(FitVertsInCameraViewError::NoIntersectionFound)
        }
    }

    /// Apply the given [`glm::TMat4`] to the given [`glm::TVec3`] to
    /// form a [`glm::TVec4`].
    fn apply_matrix_vec3_to_vec4<T: glm::RealField>(
        vec: &glm::TVec3<T>,
        mat: &glm::TMat4<T>,
    ) -> glm::TVec4<T> {
        mat * glm::vec4(vec[0], vec[1], vec[2], T::one())
    }

    /// Ray plane intersection test.
    ///
    /// Returns the distance of the intersection from the ray origin
    /// along the ray direction if the ray is not parallel with the
    /// plane and the intersection is not behind the ray.
    fn ray_plane_intersection(
        ray_origin: &glm::Vec3,
        ray_direction: &glm::Vec3,
        point_on_plane: &glm::Vec3,
        plane_normal: &glm::Vec3,
    ) -> Option<f32> {
        let d_dot_n = ray_direction.dot(plane_normal);
        if d_dot_n.abs() < f32::EPSILON {
            return None;
        }

        let t = (point_on_plane - ray_origin).dot(plane_normal) / d_dot_n;
        if t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

/// Possible errors in [`Camera::move_to_fit_verts_in_camera_view()`].
#[derive(Debug)]
pub enum FitVertsInCameraViewError {
    /// No verts provided to fit into the camera view.
    NoVertsProvided,
    /// No intersection found.
    NoIntersectionFound,
}

impl Display for FitVertsInCameraViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FitVertsInCameraViewError::NoVertsProvided => write!(f, "no verts provided"),
            FitVertsInCameraViewError::NoIntersectionFound => write!(f, "no intersection found"),
        }
    }
}

impl std::error::Error for FitVertsInCameraViewError {}

/// Direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
}

/// Camera sensor
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Sensor {
    /// sensor width
    width: f64,
    /// sensor height
    height: f64,
    /// aspect ratio of the sensor, width of the sensor with respect
    /// to the height of the aspect
    aspect_ratio: f64,
}

impl Sensor {
    /// Create a new [`Sensor`] given the width and height of the
    /// sensor.
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            aspect_ratio: width / height,
        }
    }

    /// Create a new [`Sensor`] given the width and aspect ratio of
    /// the sensor.
    pub fn from_width(width: f64, aspect_ratio: f64) -> Self {
        Self {
            width,
            height: width / aspect_ratio,
            aspect_ratio,
        }
    }

    /// Create a new [`Sensor`] given the height and aspect ratio of
    /// the sensor.
    pub fn from_height(height: f64, aspect_ratio: f64) -> Self {
        Self {
            width: height * aspect_ratio,
            height,
            aspect_ratio,
        }
    }

    /// Get sensor's width.
    pub fn get_width(&self) -> f64 {
        self.width
    }

    /// Get sensor's height.
    pub fn get_height(&self) -> f64 {
        self.height
    }

    /// Get sensor's aspect ratio.
    pub fn get_aspect_ratio(&self) -> f64 {
        self.aspect_ratio
    }

    /// Change sensor's width while keeping aspect ratio the same
    pub fn change_width(&mut self, width: f64) {
        *self = Self::from_width(width, self.get_aspect_ratio());
    }

    /// Change sensor's height while keeping aspect ratio the same
    pub fn change_height(&mut self, height: f64) {
        *self = Self::from_height(height, self.get_aspect_ratio());
    }

    /// Change sensor's aspect ratio while keeping sensor width
    /// constant. Reflects the aspect ratio change through the
    /// sensor's height
    pub fn change_aspect_ratio(&mut self, aspect_ratio: f64) {
        *self = Self::from_width(self.get_width(), aspect_ratio);
    }
}

/// Camera draw data.
pub struct CameraDrawData {
    imm: Rc<RefCell<GPUImmediate>>,
    image: Option<Rc<RefCell<TextureRGBAFloat>>>,
    alpha_value: f64,
    use_depth_for_image: bool,
}

impl CameraDrawData {
    /// Create a new [`CameraDrawData`] struct.
    pub fn new(
        imm: Rc<RefCell<GPUImmediate>>,
        image: Option<Rc<RefCell<TextureRGBAFloat>>>,
        alpha_value: f64,
        use_depth_for_image: bool,
    ) -> Self {
        Self {
            imm,
            image,
            alpha_value,
            use_depth_for_image,
        }
    }
}

impl Drawable for Camera {
    type ExtraData = CameraDrawData;
    type Error = NoSpecificDrawError;

    fn draw(&self, extra_data: &Self::ExtraData) -> Result<(), Self::Error> {
        let sensor = self.get_sensor().ok_or(NoSpecificDrawError)?;

        // Scale the camera so that the sensor width or height is 1m,
        // the other side is dependent on aspect ratio. So the sensor
        // shown (camera plane) is a constant size and the focal
        // length changes to convey the required information.
        //
        // A camera with a sensor size (width) of 36mm and a focal
        // length of 36mm will be 1m long and 1m wide in 3D space.
        let focal_length = self
            .get_focal_length()
            .expect("by this point focal length should always be available");
        // Equivalent focal length if the sensor was a 36mm sensor
        // (crop factor correction).
        let focal_length = focal_length * 36.0 / sensor.get_width();
        // Focal length required in 3D space, for a focal length of
        // 36mm it is 1m.
        let focal_length = focal_length / 36.0;
        let camera_plane_center = self.position + self.front * focal_length;

        // Sensor width of 1m.
        let horizontal = self.right / 2.0;
        // Sensor height dependent on sensor width.
        let vertical = self.up / 2.0 / sensor.get_aspect_ratio();

        let camera_plane_top_left: glm::Vec3 =
            glm::convert(camera_plane_center + -1.0 * horizontal + 1.0 * vertical);
        let camera_plane_top_right: glm::Vec3 =
            glm::convert(camera_plane_center + 1.0 * horizontal + 1.0 * vertical);
        let camera_plane_bottom_left: glm::Vec3 =
            glm::convert(camera_plane_center + -1.0 * horizontal + -1.0 * vertical);
        let camera_plane_bottom_right: glm::Vec3 =
            glm::convert(camera_plane_center + 1.0 * horizontal + -1.0 * vertical);
        let origin: glm::Vec3 = glm::convert(self.get_position());
        let vertical: glm::Vec3 = glm::convert(vertical);

        let imm = &mut extra_data.imm.borrow_mut();
        let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
            .as_ref()
            .unwrap();
        let color: glm::Vec4 = glm::vec4(0.0, 0.0, 0.0, 1.0);
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

        imm.begin(GPUPrimType::Lines, 16, smooth_color_3d_shader);

        // from origin to the plane
        draw_line(
            imm,
            &origin,
            &camera_plane_top_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_top_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_bottom_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &origin,
            &camera_plane_bottom_right,
            pos_attr,
            color_attr,
            &color,
        );

        // the plane
        draw_line(
            imm,
            &camera_plane_top_left,
            &camera_plane_top_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_top_right,
            &camera_plane_bottom_right,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_bottom_right,
            &camera_plane_bottom_left,
            pos_attr,
            color_attr,
            &color,
        );
        draw_line(
            imm,
            &camera_plane_bottom_left,
            &camera_plane_top_left,
            pos_attr,
            color_attr,
            &color,
        );

        imm.end();

        // triangle at the top
        imm.begin(GPUPrimType::Tris, 3, smooth_color_3d_shader);

        draw_triangle(
            imm,
            &camera_plane_top_left,
            &camera_plane_top_right,
            &((camera_plane_top_left + camera_plane_top_right) / 2.0 + vertical),
            pos_attr,
            color_attr,
            &color,
        );

        imm.end();

        // draw image in the camera plane
        if let Some(image) = &extra_data.image {
            if !extra_data.use_depth_for_image {
                unsafe {
                    gl::Disable(gl::DEPTH_TEST);
                }
            }

            let scale_x = (camera_plane_top_left - camera_plane_top_right).norm() as _;
            let scale_z = (camera_plane_top_left - camera_plane_bottom_left).norm() as _;
            gpu_utils::draw_plane_with_image(
                &camera_plane_center,
                &glm::vec3(scale_x, 1.0, scale_z),
                &(camera_plane_center - self.get_position()).normalize(),
                &mut image.borrow_mut(),
                extra_data.alpha_value,
                imm,
            );

            if !extra_data.use_depth_for_image {
                unsafe {
                    gl::Enable(gl::DEPTH_TEST);
                }
            }
        }

        Ok(())
    }
}

fn draw_line(
    imm: &mut GPUImmediate,
    p1: &glm::Vec3,
    p2: &glm::Vec3,
    pos_attr: usize,
    color_attr: usize,
    color: &glm::Vec4,
) {
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
}

fn draw_triangle(
    imm: &mut GPUImmediate,
    p1: &glm::Vec3,
    p2: &glm::Vec3,
    p3: &glm::Vec3,
    pos_attr: usize,
    color_attr: usize,
    color: &glm::Vec4,
) {
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p1[0], p1[1], p1[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p2[0], p2[1], p2[2]);
    imm.attr_4f(color_attr, color[0], color[1], color[2], color[3]);
    imm.vertex_3f(pos_attr, p3[0], p3[1], p3[2]);
}

/// Convert from the given vertical field of view (fov) to the
/// horizontal fov given the aspect ratio of the frustum.
///
/// Note: the fov passed to the function must be in radians.
pub fn vfov_to_hfov(vfov: f64, aspect: f64) -> f64 {
    ((vfov * 0.5).tan() * aspect).atan() * 2.0
}
