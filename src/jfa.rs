use std::convert::TryInto;

use crate::framebuffer::FrameBuffer;
use crate::gpu_immediate::GPUImmediate;
use crate::renderbuffer::RenderBuffer;
use crate::texture::TextureRGBAFloat;
use crate::{gpu_utils, shader};

/// Jump Flooding Algorithm
pub fn jfa(
    image: &mut TextureRGBAFloat,
    num_steps: usize,
    imm: &mut GPUImmediate,
) -> TextureRGBAFloat {
    unsafe {
        gl::Disable(gl::DEPTH_TEST);
    }
    let (width, height) = (image.get_width(), image.get_height());
    let mut prev_viewport_params = [0, 0, 0, 0];
    unsafe {
        gl::GetIntegerv(gl::VIEWPORT, prev_viewport_params.as_mut_ptr());
        gl::Viewport(0, 0, width.try_into().unwrap(), height.try_into().unwrap());
    }
    let jfa_initialization_shader = shader::builtins::get_jfa_initialization_shader()
        .as_ref()
        .unwrap();
    let jfa_step_shader = shader::builtins::get_jfa_step_shader().as_ref().unwrap();

    let framebuffer = FrameBuffer::new();
    let mut jfa_texture_1 = TextureRGBAFloat::new_empty(width, height);
    let mut jfa_texture_2 = TextureRGBAFloat::new_empty(width, height);
    let renderbuffer = RenderBuffer::new(width, height);
    // Initialization
    {
        framebuffer.activate(&jfa_texture_1, &renderbuffer);
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        jfa_initialization_shader.use_shader();
        jfa_initialization_shader.set_int("image\0", 31);
        image.activate(31);

        gpu_utils::render_quad_with_uv(imm, jfa_initialization_shader);
    }

    // JFA steps
    (0..num_steps).for_each(|step| {
        let render_to;
        let render_from;
        if step % 2 == 0 {
            render_from = &mut jfa_texture_1;
            render_to = &jfa_texture_2;
        } else {
            render_from = &mut jfa_texture_2;
            render_to = &jfa_texture_1;
        }

        framebuffer.activate(render_to, &renderbuffer);
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let step_size = 2.0_f32.powi((num_steps - 1 - step).try_into().unwrap());

        jfa_step_shader.use_shader();
        jfa_step_shader.set_int("image\0", 31);
        jfa_step_shader.set_float("step_size\0", step_size);
        render_from.activate(31);

        gpu_utils::render_quad_with_uv(imm, jfa_step_shader);
    });

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Viewport(
            prev_viewport_params[0],
            prev_viewport_params[1],
            prev_viewport_params[2],
            prev_viewport_params[3],
        );
    }
    FrameBuffer::activiate_default();

    if num_steps == 0 {
        jfa_texture_1
    } else if num_steps % 2 == 0 {
        jfa_texture_2
    } else {
        jfa_texture_1
    }
}

pub fn convert_to_distance(
    jfa_texture: &mut TextureRGBAFloat,
    imm: &mut GPUImmediate,
) -> TextureRGBAFloat {
    let framebuffer = FrameBuffer::new();
    let distance_texture =
        TextureRGBAFloat::new_empty(jfa_texture.get_width(), jfa_texture.get_height());
    let renderbuffer = RenderBuffer::new(jfa_texture.get_width(), jfa_texture.get_height());

    unsafe {
        gl::Disable(gl::DEPTH_TEST);
    }
    let mut prev_viewport_params = [0, 0, 0, 0];
    unsafe {
        gl::GetIntegerv(gl::VIEWPORT, prev_viewport_params.as_mut_ptr());
        gl::Viewport(
            0,
            0,
            jfa_texture.get_width().try_into().unwrap(),
            jfa_texture.get_height().try_into().unwrap(),
        );
    }
    framebuffer.activate(&distance_texture, &renderbuffer);

    let jfa_convert_to_distance_shader = shader::builtins::get_jfa_convert_to_distance_shader()
        .as_ref()
        .unwrap();
    jfa_convert_to_distance_shader.use_shader();
    jfa_convert_to_distance_shader.set_int("image\0", 31);
    jfa_texture.activate(31);

    gpu_utils::render_quad_with_uv(imm, jfa_convert_to_distance_shader);

    FrameBuffer::activiate_default();
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Viewport(
            prev_viewport_params[0],
            prev_viewport_params[1],
            prev_viewport_params[2],
            prev_viewport_params[3],
        );
    }

    distance_texture
}
