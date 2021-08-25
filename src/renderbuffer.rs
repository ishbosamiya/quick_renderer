use std::convert::TryInto;

use gl::types::GLuint;

pub struct RenderBuffer {
    gl_renderbuffer: GLuint,
}

impl RenderBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut gl_renderbuffer = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut gl_renderbuffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, gl_renderbuffer);
            gl::RenderbufferStorage(
                gl::RENDERBUFFER,
                gl::DEPTH24_STENCIL8,
                width.try_into().unwrap(),
                height.try_into().unwrap(),
            );
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        }

        Self { gl_renderbuffer }
    }

    pub fn get_gl_renderbuffer(&self) -> GLuint {
        self.gl_renderbuffer
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.gl_renderbuffer);
        }
    }
}
