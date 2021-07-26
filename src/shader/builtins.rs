use lazy_static::lazy_static;

use super::{Shader, ShaderError};

lazy_static! {
    static ref DIRECTIONAL_LIGHT: Result<Shader, ShaderError> = {
        Shader::from_strings(
            get_directional_light_vert_code(),
            get_directional_light_frag_code(),
        )
    };
}

pub fn get_directional_light_vert_code() -> &'static str {
    include_str!("../../shaders/directional_light.vert")
}

pub fn get_directional_light_frag_code() -> &'static str {
    include_str!("../../shaders/directional_light.frag")
}

pub fn get_directional_light_shader() -> &'static Result<Shader, ShaderError> {
    &DIRECTIONAL_LIGHT
}
