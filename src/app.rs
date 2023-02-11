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

    /// Run during the update loop. Guarenteed to be run once per
    /// frame.
    ///
    /// If an error is returned, the application will quit and exit
    /// out of the environment with the error provided.
    fn update(&mut self, environment: &mut Environment) -> Result<(), Box<dyn std::error::Error>>;

    /// Handle events of the window (application). There may be more
    /// than 1 event per frame.
    fn handle_window_event(
        &mut self,
        event: &glfw::WindowEvent,
        window: &mut glfw::Window,
        key_mods: &glfw::Modifiers,
    );
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
    /// Exits if `app` returns an [`Result::Err(_)`] or the window of
    /// the application closes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// Environment::new("Simple Render")?.run::<Application>()?
    /// ```
    pub fn run<T: App>(mut self, init_extra: T::InitData) -> Result<(), Error> {
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

            app.update(&mut self).map_err(|err| Error::App(err))?;

            // Swap front and back buffers
            self.window.swap_buffers();
        }

        Ok(())
    }
}
