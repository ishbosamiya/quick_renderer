//! OpenGL mesh operations.

use std::convert::TryInto;

use memoffset::offset_of;

use crate::{
    drawable::{Drawable, NoSpecificDrawError},
    glm,
    rasterize::Rasterize,
};

/// Simple vertex containing position, uv and normal information.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GLVert {
    /// Vertex position.
    pub pos: glm::Vec3,
    /// Vertex UV.
    pub uv: glm::Vec2,
    /// Vertex normal.
    pub normal: glm::Vec3,
}

impl GLVert {
    /// Create a new [`GLVert`].
    pub fn new(pos: glm::Vec3, uv: glm::Vec2, normal: glm::Vec3) -> Self {
        Self { pos, uv, normal }
    }
}

/// Index triangle. Stores the indicies of (an array of) the
/// [`GLVert`] that form the triangle.
///
/// Ordering of the indices matters if back face culling is enabled.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    /// Index of the first vertex of the triangle.
    pub i1: gl::types::GLuint,
    /// Index of the second vertex of the triangle.
    pub i2: gl::types::GLuint,
    /// Index of the third vertex of the triangle.
    pub i3: gl::types::GLuint,
}

impl Triangle {
    /// Create a new [`Triangle`].
    pub fn new(i1: gl::types::GLuint, i2: gl::types::GLuint, i3: gl::types::GLuint) -> Self {
        Self { i1, i2, i3 }
    }
}

/// OpenGL mesh.
///
/// Upon creation, the mesh is sent to the GPU for future rendering.
#[derive(Debug)]
pub struct GLMesh {
    // no need to store the verts and indices, currently there is no
    // way to update the verts or indices thus does not need to be
    // stored on the CPU as well.
    //
    // verts: Vec<GLVert>,
    // triangles: Vec<Triangle>,
    num_triangles: usize,

    vao: Option<gl::types::GLuint>,
    vbo: Option<gl::types::GLuint>,
    ebo: Option<gl::types::GLuint>,
}

impl Drop for GLMesh {
    fn drop(&mut self) {
        if self.vao.is_some() {
            assert!(self.vbo.is_some());
            assert!(self.ebo.is_some());
            self.cleanup_opengl();
        }
    }
}

impl Rasterize for GLMesh {
    fn cleanup_opengl(&mut self) {
        unsafe {
            gl::BindVertexArray(self.vao.unwrap());
            gl::DeleteBuffers(2, [self.vbo.unwrap(), self.ebo.unwrap()].as_ptr());
        }
        self.vao = None;
        self.vbo = None;
        self.ebo = None;
    }
}

impl GLMesh {
    /// Create a new [`GLMesh`].
    pub fn new(verts: &[GLVert], triangles: &[Triangle]) -> Self {
        Self::setup(verts, triangles)
    }

    /// Setup the [`GLMesh`] for rendering.
    fn setup(verts: &[GLVert], triangles: &[Triangle]) -> Self {
        let (vao, vbo, ebo) = unsafe {
            let mut vao: gl::types::GLuint = 0;
            let mut vbo: gl::types::GLuint = 0;
            let mut ebo: gl::types::GLuint = 0;
            // generate the buffers needed
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            (vao, vbo, ebo)
        };

        if vao == 0 || vbo == 0 || ebo == 0 {
            panic!("vao, vbo, or ebo couldn't be initialized");
        }

        unsafe {
            gl::BindVertexArray(vao);

            // bind verts array
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(verts).try_into().unwrap(),
                verts.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            // bind indices array
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (3 * triangles.len() * std::mem::size_of::<gl::types::GLuint>())
                    .try_into()
                    .unwrap(),
                triangles.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );

            // positions at attribute location 0
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, pos) as *const gl::types::GLvoid,
            );
            // uvs at attribute location 2
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, uv) as *const gl::types::GLvoid,
            );
            // normals at attribute location 1
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                std::mem::size_of::<GLVert>().try_into().unwrap(),
                offset_of!(GLVert, normal) as *const gl::types::GLvoid,
            );

            gl::BindVertexArray(0);
        }

        Self {
            num_triangles: triangles.len(),
            vao: Some(vao),
            vbo: Some(vbo),
            ebo: Some(ebo),
        }
    }

    /// Get the number of triangles of the mesh.
    pub fn num_triangles(&self) -> usize {
        self.num_triangles
    }
}

impl Drawable for GLMesh {
    type ExtraData = ();
    type Error = NoSpecificDrawError;

    fn draw(&self, _extra_data: &Self::ExtraData) -> Result<(), Self::Error> {
        unsafe {
            gl::BindVertexArray(self.vao.unwrap());
            gl::DrawElements(
                gl::TRIANGLES,
                (3 * self.num_triangles).try_into().unwrap(),
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
            gl::BindVertexArray(0);
        }
        Ok(())
    }

    fn draw_wireframe(&self, _extra_data: &Self::ExtraData) -> Result<(), Self::Error> {
        unreachable!("no wireframe support for GLMesh")
    }
}
