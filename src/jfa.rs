use std::convert::TryInto;

use crate::framebuffer::FrameBuffer;
use crate::gpu_immediate::GPUImmediate;
use crate::renderbuffer::RenderBuffer;
use crate::texture::TextureRGBAFloat;
use crate::{gpu_utils, shader};

/// Jump Flooding Algorithm.
///
/// For the given [`TextureRGBAFloat`], jump flooding of the areas of
/// the image that have their `r + g` values greater than 0.0 is
/// done. The `b and a` values are preserved thus can be used to store
/// additional information in the image.
///
/// # Note
///
/// * This is slow at the moment since everytime this function is
/// called, new textures are allocated and then destroyed at the end
/// of it. This is mainly for testing purposes and a way to show how
/// it could possibly be implemented.
///
/// * It is important to reset the framebuffer state to whatever is
/// necessary since an internal framebuffer is made active during the
/// execution.
pub fn jfa(
    image: &mut TextureRGBAFloat,
    num_steps: usize,
    imm: &mut GPUImmediate,
) -> TextureRGBAFloat {
    let (width, height) = (image.get_width(), image.get_height());
    let mut prev_viewport_params = [0, 0, 0, 0];
    let prev_depth_enable = unsafe { gl::IsEnabled(gl::DEPTH_TEST) } != 0;
    let prev_blend_enable = unsafe { gl::IsEnabled(gl::BLEND) } != 0;
    unsafe {
        gl::GetIntegerv(gl::VIEWPORT, prev_viewport_params.as_mut_ptr());
        gl::Viewport(0, 0, width.try_into().unwrap(), height.try_into().unwrap());
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::BLEND);
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
        framebuffer.activate(&mut jfa_texture_1, &renderbuffer);

        // no need to clear the framebuffer since blending is turned
        // off, it will just overwrite the pixels

        jfa_initialization_shader.use_shader();
        jfa_initialization_shader.set_int("u_image\0", 31);
        image.activate(31);

        gpu_utils::draw_screen_quad_with_uv(imm, jfa_initialization_shader);
    }

    // JFA steps
    (0..num_steps).for_each(|step| {
        let (render_from, render_to) = if step % 2 == 0 {
            (&mut jfa_texture_1, &mut jfa_texture_2)
        } else {
            (&mut jfa_texture_2, &mut jfa_texture_1)
        };

        framebuffer.activate(render_to, &renderbuffer);

        // no need to clear the framebuffer since blending is turned
        // off, it will just overwrite the pixels

        let step_size = 2.0_f32.powi((num_steps - 1 - step).try_into().unwrap());

        jfa_step_shader.use_shader();
        jfa_step_shader.set_int("u_image\0", 31);
        jfa_step_shader.set_float("u_step_size\0", step_size);
        render_from.activate(31);

        gpu_utils::draw_screen_quad_with_uv(imm, jfa_step_shader);
    });

    unsafe {
        gl::Viewport(
            prev_viewport_params[0],
            prev_viewport_params[1],
            prev_viewport_params[2],
            prev_viewport_params[3],
        );

        if prev_depth_enable {
            gl::Enable(gl::DEPTH_TEST);
        }
        if prev_blend_enable {
            gl::Enable(gl::BLEND);
        }
    }

    if num_steps % 2 == 0 {
        jfa_texture_1
    } else {
        jfa_texture_2
    }
}

pub fn convert_to_distance(
    jfa_texture: &mut TextureRGBAFloat,
    imm: &mut GPUImmediate,
) -> TextureRGBAFloat {
    let framebuffer = FrameBuffer::new();
    let mut distance_texture =
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
    framebuffer.activate(&mut distance_texture, &renderbuffer);

    let jfa_convert_to_distance_shader = shader::builtins::get_jfa_convert_to_distance_shader()
        .as_ref()
        .unwrap();
    jfa_convert_to_distance_shader.use_shader();
    jfa_convert_to_distance_shader.set_int("u_image\0", 31);
    jfa_texture.activate(31);

    gpu_utils::draw_screen_quad_with_uv(imm, jfa_convert_to_distance_shader);

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
