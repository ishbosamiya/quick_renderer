//! A simple interface to create a quick GUI application. It is
//! limited in functionality thus useful only to create simple
//! prototypes. It can serve as a good base to understand how the
//! application can be setup to create something more complex.

use std::{fmt::Display, sync::mpsc::Receiver};

use glfw::{self, Context};

use crate::fps::FPS;

#[derive(Debug)]
pub enum Error {
    GlfwInit(glfw::InitError),
    Glfw(glfw::Error),
    GlfwWindowCreation,
    App(Box<dyn std::error::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GlfwInit(err) => write!(f, "{}", err),
            Error::Glfw(err) => write!(f, "Glfw: {}", err),
            Error::GlfwWindowCreation => write!(f, "Glfw window creation"),
            Error::App(err) => write!(f, "App: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<glfw::InitError> for Error {
    fn from(err: glfw::InitError) -> Self {
        Self::GlfwInit(err)
    }
}

impl From<glfw::Error> for Error {
    fn from(err: glfw::Error) -> Self {
        Self::Glfw(err)
    }
}

/// Implementing this trait is the way to create a simple application.
pub trait App {
    /// Type of data to pass to [`Self::init()`].
    type InitData;

    /// The initialization of the application. This is the entry point
    /// for the application through the creation of the application.
    ///
    /// Initialization of certain components of the application
    /// sometimes requires the [`Environment`] to already be
    /// initialized, which is given through `environment`.
    ///
    /// If an error is returned, the application will quit and exit
    /// out of the environment with the error provided.
    fn init(
        environment: &mut Environment,
        extra: Self::InitData,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: std::marker::Sized;

    /// Type of data that is returned when the [`App`] exits.
    type ExitData;

    /// Run during the update loop. Guarenteed to be run once per
    /// frame.
    ///
    /// The application exits if [`Ok`]`(`[`MaybeContinue::Exit`]`)`
    /// or [`Err`] is returned.
    fn update(
        &mut self,
        environment: &mut Environment,
    ) -> Result<MaybeContinue<Self::ExitData>, Box<dyn std::error::Error>>;

    /// Handle events of the window (application). There may be more
    /// than 1 event per frame.
    fn handle_window_event(
        &mut self,
        event: &glfw::WindowEvent,
        window: &mut glfw::Window,
        key_mods: &glfw::Modifiers,
    );
}

/// Maybe the app should continue running.
pub enum MaybeContinue<T> {
    /// Continue running the app.
    Continue,
    /// Exit the app with some data.
    Exit(T),
}

/// Environment of the application that handles the boiler plate code
/// to create a GUI application.
pub struct Environment {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    events_receiver: Receiver<(f64, glfw::WindowEvent)>,
    pub fps: FPS,
}

impl Environment {
    /// Create a new environment.
    ///
    /// Spawns a new window with an OpenGL context and window title as
    /// `application_name`.
    pub fn new(application_name: &str) -> Result<Self, Error> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        // creating window
        let (mut window, events_receiver) = glfw
            .create_window(1280, 720, application_name, glfw::WindowMode::Windowed)
            .ok_or(Error::GlfwWindowCreation)?;

        // setup bunch of polling data
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_scroll_polling(true);
        window.set_char_polling(true);
        window.make_current();

        gl::load_with(|symbol| window.get_proc_address(symbol));

        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::MULTISAMPLE);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }

        let fps = FPS::default();

        Ok(Self {
            glfw,
            window,
            events_receiver,
            fps,
        })
    }

    /// Run the environment with the given [`App`]. The [`App`] is
    /// given through a generic argument.
    ///
    /// Exits if the app returns [`Ok`]`(`[`MaybeContinue::Exit`]`)`
    /// or [`Err`]`(_)` in its [`App::update()`] routine. Also exits
    /// if the window of the application closes.
    ///
    /// Upon the [`App`] exiting, the [`App`] is returned. If the
    /// [`App`] exited with [`MaybeContinue::Exit`], the given data is
    /// returned too.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Environment::new("Simple Render")?.run::<Application>()?
    /// ```
    pub fn run<T: App>(
        mut self,
        init_extra: T::InitData,
    ) -> Result<(T, Option<T::ExitData>), Error> {
        let mut key_mods = glfw::Modifiers::empty();

        let mut app = T::init(&mut self, init_extra).map_err(|err| Error::App(err))?;

        while !self.window.should_close() {
            self.glfw.poll_events();

            let events_receiver = &self.events_receiver;
            let window = &mut self.window;

            glfw::flush_messages(events_receiver).for_each(|(_, event)| {
                match event {
                    glfw::WindowEvent::Key(_, _, glfw::Action::Press, mods) => key_mods |= mods,
                    glfw::WindowEvent::Key(_, _, glfw::Action::Release, mods) => key_mods &= !mods,
                    glfw::WindowEvent::CharModifiers(_, mods) => key_mods |= mods,
                    glfw::WindowEvent::MouseButton(_, glfw::Action::Press, mods) => {
                        key_mods |= mods
                    }
                    glfw::WindowEvent::MouseButton(_, glfw::Action::Release, mods) => {
                        key_mods &= !mods
                    }
                    _ => {}
                }

                app.handle_window_event(&event, window, &key_mods);
            });

            match app.update(&mut self).map_err(|err| Error::App(err))? {
                MaybeContinue::Continue => {
                    // continue to next frame
                }
                MaybeContinue::Exit(data) => {
                    return Ok((app, Some(data)));
                }
            }

            // Swap front and back buffers
            self.window.swap_buffers();
        }

        Ok((app, None))
    }
}

/// Settings for the [`Environment`].
pub struct EnvironmentSettings {
    /// Width and height of the initial window.
    pub window_dimensions: (u32, u32),

    /// Enable pos polling?
    pub pos_polling: bool,
    /// Enable size_polling?
    pub size_polling: bool,
    /// Enable close polling?
    pub close_polling: bool,
    /// Enable refresh polling?
    pub refresh_polling: bool,
    /// Enable focus polling?
    pub focus_polling: bool,
    /// Enable iconify polling?
    pub iconify_polling: bool,
    /// Enable framebuffer size polling?
    pub framebuffer_size_polling: bool,
    /// Enable key polling?
    pub key_polling: bool,
    /// Enable char polling?
    pub char_polling: bool,
    /// Enable char mods polling?
    pub char_mods_polling: bool,
    /// Enable mouse button polling?
    pub mouse_button_polling: bool,
    /// Enable cursor pos polling?
    pub cursor_pos_polling: bool,
    /// Enable cursor enter polling?
    pub cursor_enter_polling: bool,
    /// Enable scroll polling?
    pub scroll_polling: bool,
    /// Enable drag and drop polling?
    pub drag_and_drop_polling: bool,
    /// Enable maximize polling?
    pub maximize_polling: bool,
    /// Enable content scale polling?
    pub content_scale_polling: bool,

    /// Specifies the client API major and minor version that the
    /// created context must be compatible with.
    ///
    /// Window creation will fail if the resulting OpenGL version is
    /// less than the one requested.
    pub context_version: (u32, u32),
    /// [`glfw::OpenGLProfileHint`].
    pub opengl_profile_hint: glfw::OpenGlProfileHint,

    /// Load OpenGL?
    pub load_opengl: bool,
}
