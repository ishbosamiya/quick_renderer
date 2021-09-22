use lazy_static::lazy_static;

use crate::glm;
use crate::gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode};
use crate::shader::Shader;

lazy_static! {
    static ref PLANE_VERT_LIST_F32: Vec<(glm::Vec3, glm::Vec2)> = vec![
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(-1.0, 1.0, 0.0), glm::vec2(0.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(1.0, -1.0, 0.0), glm::vec2(1.0, 0.0)),
    ];
    static ref PLANE_VERT_LIST_F64: Vec<(glm::DVec3, glm::DVec2)> = vec![
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(-1.0, 1.0, 0.0), glm::vec2(0.0, 1.0)),
        (glm::vec3(-1.0, -1.0, 0.0), glm::vec2(0.0, 0.0)),
        (glm::vec3(1.0, 1.0, 0.0), glm::vec2(1.0, 1.0)),
        (glm::vec3(1.0, -1.0, 0.0), glm::vec2(1.0, 0.0)),
    ];
}

pub fn get_plane_vert_list_f32() -> &'static Vec<(glm::Vec3, glm::Vec2)> {
    &PLANE_VERT_LIST_F32
}

pub fn get_plane_vert_list_f64() -> &'static Vec<(glm::DVec3, glm::DVec2)> {
    &PLANE_VERT_LIST_F64
}

pub fn render_quad_with_uv(imm: &mut GPUImmediate, shader: &Shader) {
    let format = imm.get_cleared_vertex_format();
    let pos_attr = format.add_attribute(
        "in_pos\0".to_string(),
        GPUVertCompType::F32,
        3,
        GPUVertFetchMode::Float,
    );
    let uv_attr = format.add_attribute(
        "in_uv\0".to_string(),
        GPUVertCompType::F32,
        2,
        GPUVertFetchMode::Float,
    );

    imm.begin(GPUPrimType::Tris, 6, shader);

    get_plane_vert_list_f32().iter().for_each(|(pos, uv)| {
        imm.attr_2f(uv_attr, uv[0], uv[1]);
        imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);
    });

    imm.end();
}

pub fn render_quad(imm: &mut GPUImmediate, shader: &Shader) {
    let format = imm.get_cleared_vertex_format();
    let pos_attr = format.add_attribute(
        "in_pos\0".to_string(),
        GPUVertCompType::F32,
        3,
        GPUVertFetchMode::Float,
    );

    imm.begin(GPUPrimType::Tris, 6, shader);

    get_plane_vert_list_f32().iter().for_each(|(pos, _uv)| {
        imm.vertex_3f(pos_attr, pos[0], pos[1], pos[2]);
    });

    imm.end();
}
