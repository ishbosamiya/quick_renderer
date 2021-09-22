use lazy_static::lazy_static;

use crate::drawable::Drawable;
use crate::gpu_immediate::{GPUImmediate, GPUPrimType, GPUVertCompType, GPUVertFetchMode};
use crate::mesh::MeshDrawData;
use crate::shader::Shader;
use crate::{glm, mesh, shader};

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

/// Draws a smooth sphere at the given position with the given radius.
///
/// This is a fairly expensive draw call since it traces rays from all
/// the fragments of the render target to test if it has intersected
/// with the sphere to set the fragment's color depending on whether
/// inside or outside of the sphere is hit.
///
/// For a less expensive draw call (for sphere at cover a small
/// portion of the render target) use `draw_sphere_at()`.
pub fn draw_smooth_sphere_at(
    pos: glm::DVec3,
    radius: f64,
    outside_color: glm::Vec4,
    inside_color: glm::Vec4,
    imm: &mut GPUImmediate,
) {
    let smooth_sphere_shader = shader::builtins::get_smooth_sphere_shader()
        .as_ref()
        .unwrap();

    smooth_sphere_shader.use_shader();
    smooth_sphere_shader.set_vec4("outside_color\0", &outside_color);
    smooth_sphere_shader.set_vec4("inside_color\0", &inside_color);
    smooth_sphere_shader.set_vec3("sphere_center\0", &glm::convert(pos));
    smooth_sphere_shader.set_float("sphere_radius\0", radius as _);

    render_quad(imm, smooth_sphere_shader);
}

/// Draws a sphere at the given position with the given radius.
///
/// Draws an ico sphere and thus is not smooth. It is good for spheres
/// that cover a small portion of the render target. For smooth
/// spheres that cover a large portion of the render target use
/// `draw_smooth_sphere_at()`.
pub fn draw_sphere_at(pos: &glm::DVec3, radius: f64, color: glm::Vec4, imm: &mut GPUImmediate) {
    let smooth_color_3d_shader = shader::builtins::get_smooth_color_3d_shader()
        .as_ref()
        .unwrap();
    smooth_color_3d_shader.use_shader();
    smooth_color_3d_shader.set_mat4(
        "model\0",
        &glm::convert(glm::scale(
            &glm::translate(&glm::identity(), pos),
            &glm::vec3(radius, radius, radius),
        )),
    );

    let ico_sphere = mesh::builtins::get_ico_sphere_subd_01();

    ico_sphere
        .draw(&mut MeshDrawData::new(
            imm,
            mesh::MeshUseShader::SmoothColor3D,
            Some(color),
        ))
        .unwrap();
}
