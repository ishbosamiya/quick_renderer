use lazy_static::lazy_static;

use super::{Shader, ShaderError};

macro_rules! load_builtin_shader {
    ( $get_shader:ident ; $get_vert_code:ident ; $get_frag_code:ident ; $vert_location:tt ; $frag_location:tt ; $static_name:ident ) => {
        lazy_static! {
            static ref $static_name: Result<Shader, ShaderError> =
                { Shader::from_strings($get_vert_code(), $get_frag_code(),) };
        }

        pub fn $get_vert_code() -> &'static str {
            include_str!($vert_location)
        }

        pub fn $get_frag_code() -> &'static str {
            include_str!($frag_location)
        }

        pub fn $get_shader() -> &'static Result<Shader, ShaderError> {
            &$static_name
        }
    };
}

load_builtin_shader!(
    get_directional_light_shader;
    get_directional_light_vert_code;
    get_directional_light_frag_code;
    "../../shaders/directional_light.vert";
    "../../shaders/directional_light.frag";
    DIRECTIONAL_LIGHT
);

load_builtin_shader!(
    get_smooth_color_3d_shader;
    get_smooth_color_3d_vert_code;
    get_smooth_color_3d_frag_code;
    "../../shaders/shader_3D_smooth_color.vert";
    "../../shaders/shader_3D_smooth_color.frag";
    SMOOTH_COLOR_3D
);

load_builtin_shader!(
    get_infinite_grid_shader;
    get_infinite_grid_vert_code;
    get_infinite_grid_frag_code;
    "../../shaders/infinite_grid.vert";
    "../../shaders/infinite_grid.frag";
    INFINITE_GRID
);
