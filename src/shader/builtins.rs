use lazy_static::lazy_static;
use paste::paste;

use super::{Shader, ShaderError};
use crate::camera::Camera;
use crate::glm;

/// Setup a static ref of a [`String`] by including file at the given
/// location or optionally with `NO_INCLUDE`, load the file at run
/// time.
macro_rules! setup_static_ref_string {
    ( $location:literal ) => {
        include_str!($location).to_string()
    };

    ( $location:literal NO_INCLUDE ) => {
        std::fs::read_to_string(
            std::path::Path::new(file!())
                .parent()
                .unwrap()
                .join($location),
        )
        .unwrap()
    };
}

/// Load the shader code into the executable and provide functions to
/// access the [`Shader`] and it's the code (as a reference to
/// [`str`]).
///
/// The arguments must be in the order
///
/// 1. name of function that gets the [`Shader`].
///
/// 2. name of function that gets the vertex shader code.
///
/// 3. name of function that gets the fragment shader code.
///
/// 4. path to vertex shader code relative to the file containing this
/// macro invocation.
///
/// 5. path to fragment shader code relative to the file containing
/// this macro invocation.
///
/// 6. name of the static reference to the Shader result.
///
/// 7. optionally, `NO_INCLUDE` to indicate that the shader code
/// should be included during compilation (using [`include_str`] but
/// be loaded into memory at run time. NOTE: this should be used only
/// for debugging purposes, to make it easier to modify and test the
/// shader code without recompiling the executable with every change
/// (still requires the executable to be reexecuted post changing the
/// shader code, it only removes the need for Rust to recompile
/// everything).
#[macro_export]
macro_rules! load_builtin_shader {
    ( $get_shader:ident ; $get_vert_code:ident ; $get_frag_code:ident ; $vert_location:literal ; $frag_location:literal ; $static_name:ident ; $($no_include:tt)? ) => {
        lazy_static! {
            static ref $static_name: Result<Shader, ShaderError> =
                { Shader::from_strings($get_vert_code(), $get_frag_code(),) };
        }

        paste! {
            lazy_static! {
                static ref [<$static_name _VERT_CODE>]: String = {
                    setup_static_ref_string!( $vert_location $($no_include)* )
                };

                static ref [<$static_name _FRAG_CODE>]: String = {
                    setup_static_ref_string!( $frag_location $($no_include)* )
                };
            }
        }

        pub fn $get_vert_code() -> &'static str {
            paste! {
                &[<$static_name _VERT_CODE>]
            }
        }

        pub fn $get_frag_code() -> &'static str {
            paste! {
                &[<$static_name _FRAG_CODE>]
            }
        }

        pub fn $get_shader() -> &'static Result<Shader, ShaderError> {
            &$static_name
        }
    };

    ( $get_shader:ident ; $get_vert_code:ident ; $get_frag_code:ident ; $vert_location:literal ; $frag_location:literal ; $static_name:ident $(;)? ) => {
        load_builtin_shader!($get_shader; $get_vert_code; $get_frag_code; $vert_location; $frag_location; $static_name;);
    };
}

/// An easy way to load the shader code into the executable and
/// provide functions to access the [`Shader`] and it's code (as a
/// reference to [`str`].
///
/// Internally makes a call to [`load_builtin_shader`] with automatic
/// expansion of many parameters.
#[macro_export]
macro_rules! load_builtin_shader_easy {
    ( $name:ident ; $vert_location:literal ; $frag_location:literal $(;)? ) => {
        paste! {
            load_builtin_shader!([<get_ $name _shader>]; [<get_ $name _vert_code>]; [<get_ $name _frag_code>]; $vert_location; $frag_location; [<$name:upper>] );
        }
    };

    ( $name:ident ; $vert_location:literal ; $frag_location:literal ; $no_include:tt ) => {
        paste! {
            load_builtin_shader!([<get_ $name _shader>]; [<get_ $name _vert_code>]; [<get_ $name _frag_code>]; $vert_location; $frag_location; [<$name:upper>] ; $no_include);
        }
    };
}

load_builtin_shader_easy!(
    directional_light;
    "../../shaders/directional_light.vert";
    "../../shaders/directional_light.frag";
);

load_builtin_shader_easy!(
    smooth_color_3d;
    "../../shaders/shader_3D_smooth_color.vert";
    "../../shaders/shader_3D_smooth_color.frag"
);

load_builtin_shader_easy!(
    infinite_grid;
    "../../shaders/infinite_grid.vert";
    "../../shaders/infinite_grid.frag"
);

load_builtin_shader_easy!(
    face_orientation;
    "../../shaders/face_orientation.vert";
    "../../shaders/face_orientation.frag"
);

load_builtin_shader_easy!(
    flat_texture;
    "../../shaders/flat_texture.vert";
    "../../shaders/flat_texture.frag"
);

load_builtin_shader_easy!(
    jfa_initialization;
    "../../shaders/jfa_initialization.vert";
    "../../shaders/jfa_initialization.frag"
);

load_builtin_shader_easy!(
    jfa_step;
    "../../shaders/jfa_step.vert";
    "../../shaders/jfa_step.frag"
);

load_builtin_shader_easy!(
    jfa_convert_to_distance;
    "../../shaders/jfa_convert_to_distance.vert";
    "../../shaders/jfa_convert_to_distance.frag"
);

load_builtin_shader_easy!(
    smooth_sphere;
    "../../shaders/smooth_sphere.vert";
    "../../shaders/smooth_sphere.frag"
);

pub fn display_uniform_and_attribute_info() {
    {
        let directional_light_shader = get_directional_light_shader().as_ref().unwrap();

        println!(
            "directional_light: uniforms: {:?} attributes: {:?}",
            directional_light_shader.get_uniforms(),
            directional_light_shader.get_attributes(),
        );
    }

    {
        let smooth_color_3d_shader = get_smooth_color_3d_shader().as_ref().unwrap();

        println!(
            "smooth_color_3d: uniforms: {:?} attributes: {:?}",
            smooth_color_3d_shader.get_uniforms(),
            smooth_color_3d_shader.get_attributes(),
        );
    }

    {
        let infinite_grid_shader = get_infinite_grid_shader().as_ref().unwrap();

        println!(
            "infinite_grid: uniforms: {:?} attributes: {:?}",
            infinite_grid_shader.get_uniforms(),
            infinite_grid_shader.get_attributes(),
        );
    }

    {
        let face_orientation_shader = get_face_orientation_shader().as_ref().unwrap();

        println!(
            "face_orientation: uniforms: {:?} attributes: {:?}",
            face_orientation_shader.get_uniforms(),
            face_orientation_shader.get_attributes(),
        );
    }

    {
        let flat_texture_shader = get_flat_texture_shader().as_ref().unwrap();

        println!(
            "flat_texture: uniforms: {:?} attributes: {:?}",
            flat_texture_shader.get_uniforms(),
            flat_texture_shader.get_attributes(),
        );
    }

    {
        let jfa_initialization_shader = get_jfa_initialization_shader().as_ref().unwrap();

        println!(
            "jfa_initialization: uniforms: {:?} attributes: {:?}",
            jfa_initialization_shader.get_uniforms(),
            jfa_initialization_shader.get_attributes(),
        );
    }

    {
        let jfa_step_shader = get_jfa_step_shader().as_ref().unwrap();

        println!(
            "jfa_step: uniforms: {:?} attributes: {:?}",
            jfa_step_shader.get_uniforms(),
            jfa_step_shader.get_attributes(),
        );
    }

    {
        let jfa_convert_to_distance_shader = get_jfa_convert_to_distance_shader().as_ref().unwrap();

        println!(
            "jfa_convert_to_distance: uniforms: {:?} attributes: {:?}",
            jfa_convert_to_distance_shader.get_uniforms(),
            jfa_convert_to_distance_shader.get_attributes(),
        );
    }

    {
        let smooth_sphere_shader = get_smooth_sphere_shader().as_ref().unwrap();

        println!(
            "smooth_sphere: uniforms: {:?} attributes: {:?}",
            smooth_sphere_shader.get_uniforms(),
            smooth_sphere_shader.get_attributes(),
        );
    }
}

pub fn setup_shaders(camera: &Camera, window_width: usize, window_height: usize) {
    let projection_matrix =
        &glm::convert(camera.get_perspective_projection_matrix(window_width, window_height));
    let view_matrix = &glm::convert(camera.get_view_matrix());

    {
        let directional_light_shader = get_directional_light_shader().as_ref().unwrap();

        directional_light_shader.use_shader();
        directional_light_shader.set_mat4("projection\0", projection_matrix);
        directional_light_shader.set_mat4("view\0", view_matrix);
        directional_light_shader.set_mat4("model\0", &glm::identity());
        directional_light_shader.set_vec3("viewPos\0", &glm::convert(camera.get_position()));
        directional_light_shader.set_vec3("material.color\0", &glm::vec3(0.3, 0.2, 0.7));
        directional_light_shader.set_vec3("material.specular\0", &glm::vec3(0.3, 0.3, 0.3));
        directional_light_shader.set_float("material.shininess\0", 4.0);
        directional_light_shader.set_vec3("light.direction\0", &glm::vec3(-0.7, -1.0, -0.7));
        directional_light_shader.set_vec3("light.ambient\0", &glm::vec3(0.3, 0.3, 0.3));
        directional_light_shader.set_vec3("light.diffuse\0", &glm::vec3(1.0, 1.0, 1.0));
        directional_light_shader.set_vec3("light.specular\0", &glm::vec3(1.0, 1.0, 1.0));
    }

    {
        let smooth_color_3d_shader = get_smooth_color_3d_shader().as_ref().unwrap();

        smooth_color_3d_shader.use_shader();
        smooth_color_3d_shader.set_mat4("projection\0", projection_matrix);
        smooth_color_3d_shader.set_mat4("view\0", view_matrix);
        smooth_color_3d_shader.set_mat4("model\0", &glm::identity());
    }

    {
        let infinite_grid_shader = get_infinite_grid_shader().as_ref().unwrap();

        infinite_grid_shader.use_shader();
        infinite_grid_shader.set_mat4("projection\0", projection_matrix);
        infinite_grid_shader.set_mat4("view\0", view_matrix);
    }

    {
        let face_orientation_shader = get_face_orientation_shader().as_ref().unwrap();

        face_orientation_shader.use_shader();
        face_orientation_shader.set_mat4("projection\0", projection_matrix);
        face_orientation_shader.set_mat4("view\0", view_matrix);
        face_orientation_shader.set_mat4("model\0", &glm::identity());
        face_orientation_shader.set_vec4("color_face_front\0", &glm::vec4(0.0, 0.0, 1.0, 1.0));
        face_orientation_shader.set_vec4("color_face_back\0", &glm::vec4(1.0, 0.0, 0.0, 1.0));
    }

    {
        let flat_texture_shader = get_flat_texture_shader().as_ref().unwrap();

        flat_texture_shader.use_shader();
        flat_texture_shader.set_mat4("projection\0", projection_matrix);
        flat_texture_shader.set_mat4("view\0", view_matrix);
        flat_texture_shader.set_mat4("model\0", &glm::identity());
    }

    {
        let jfa_initialization_shader = get_jfa_initialization_shader().as_ref().unwrap();

        jfa_initialization_shader.use_shader();
    }

    {
        let jfa_step_shader = get_jfa_step_shader().as_ref().unwrap();

        jfa_step_shader.use_shader();
    }

    {
        let jfa_convert_to_distance_shader = get_jfa_convert_to_distance_shader().as_ref().unwrap();

        jfa_convert_to_distance_shader.use_shader();
    }

    {
        let smooth_sphere_shader = get_smooth_sphere_shader().as_ref().unwrap();

        smooth_sphere_shader.use_shader();
        smooth_sphere_shader.set_mat4("projection\0", projection_matrix);
        smooth_sphere_shader.set_mat4("view\0", view_matrix);
    }
}
