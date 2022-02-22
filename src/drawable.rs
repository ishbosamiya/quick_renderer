use std::fmt::Display;

/// Any object that is drawable must implement this trait.
///
/// TODO: need to create a more complex system that allows for
/// multiple passes and shader selection without having to implement
/// hacks.
///
/// TODO: ExtraData should not be required, all data should be self
/// containted. Will need to analyze a couple of game engines to
/// understand the best approach to this.
///
/// TODO: it might make sense to not have Error be a user defined, can
/// make it harder to access the objects as a generic Drawable.
pub trait Drawable {
    /// Extra data required by the object for it's drawing purposes.
    ///
    /// # Note
    ///
    /// Any extra data that does needs to mutate must be wrapped in a
    /// structure that allows for interior mutablity such as
    /// [`std::cell::RefCell`] (Mutex or RwLock is not recommended
    /// since most draw calls don't support multithreading anyway).
    type ExtraData;
    /// Any error that might be produced while drawing the object.
    type Error: std::error::Error;

    fn draw(&self, extra_data: &Self::ExtraData) -> Result<(), Self::Error>;
    fn draw_wireframe(&self, _extra_data: &Self::ExtraData) -> Result<(), Self::Error> {
        eprintln!("error: draw_wireframe() not implemented but called");
        Ok(())
    }
}

/// When no specific draw error is possible, use this type for
/// Drawable::Error.
#[derive(Debug)]
pub struct NoSpecificDrawError;

impl Display for NoSpecificDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No Specific Draw Error")
    }
}

impl std::error::Error for NoSpecificDrawError {}
