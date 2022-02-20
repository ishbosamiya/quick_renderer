/// Any struct/enum that stores any OpenGL related memory (buffers,
/// textures, render targets, etc.) must implement [`Rasterize`] to
/// ensure appropriate cleanup can be done. It is not a solution that
/// just magically helps with the cleanup but makes it easier to
/// access and handle it.
pub trait Rasterize {
    /// Cleanup any OpenGL related data (buffers, etc.). Often is
    /// similar to [`Drop::drop()`] but it should ensure that
    /// [`Drop::drop()`] itself does not cause a double free. A good
    /// approach is to have the OpenGL data that would need to be
    /// freed wrapped by an [`Option`] and [`Rasterize::cleanup_opengl()`] should
    /// cleanup the data from the GPU then set this data to
    /// [`None`]. [`Drop::drop()`] would run [`Rasterize::cleanup_opengl()`] only
    /// if the data is not [`None`].
    fn cleanup_opengl(&mut self);
}
