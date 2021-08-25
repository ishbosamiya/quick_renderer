use gl::types::GLuint;

use crate::renderbuffer::RenderBuffer;
use crate::texture::TextureRGBAFloat;

pub struct FrameBuffer {
    gl_framebuffer: GLuint,
}

impl FrameBuffer {
    pub fn activiate_default() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn new() -> Self {
        let mut gl_framebuffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut gl_framebuffer);
        }

        Self { gl_framebuffer }
    }

    pub fn activate(&self, texture: &TextureRGBAFloat, renderbuffer: &RenderBuffer) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.gl_framebuffer);
        }

        unsafe {
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.get_gl_tex(),
                0,
            );
        }

        // renderbuffer so that depth testing and such can work
        unsafe {
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                renderbuffer.get_gl_renderbuffer(),
            );
        }

        let status;
        unsafe {
            status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
        }
        if status != gl::FRAMEBUFFER_COMPLETE {
            eprintln!("error: framebuffer not complete!");
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.gl_framebuffer);
        }
    }
}
