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
    pub fn new(application_name: &str, settings: &EnvironmentSettings) -> Result<Self, Error> {
        let mut glfw = glfw::init(glfw::fail_on_errors)?;

        glfw.window_hint(glfw::WindowHint::ContextVersion(
            settings.context_version.0,
            settings.context_version.1,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            settings.opengl_profile_hint,
        ));

        // creating window
        let (mut window, events_receiver) = glfw
            .create_window(
                settings.window_dimensions.0,
                settings.window_dimensions.1,
                application_name,
                glfw::WindowMode::Windowed,
            )
            .ok_or(Error::GlfwWindowCreation)?;

        // setup bunch of polling data
        window.set_pos_polling(settings.pos_polling);
        window.set_size_polling(settings.size_polling);
        window.set_close_polling(settings.close_polling);
        window.set_refresh_polling(settings.refresh_polling);
        window.set_focus_polling(settings.focus_polling);
        window.set_iconify_polling(settings.iconify_polling);
        window.set_framebuffer_size_polling(settings.framebuffer_size_polling);
        window.set_key_polling(settings.key_polling);
        window.set_char_polling(settings.char_polling);
        window.set_char_mods_polling(settings.char_mods_polling);
        window.set_mouse_button_polling(settings.mouse_button_polling);
        window.set_cursor_pos_polling(settings.cursor_pos_polling);
        window.set_cursor_enter_polling(settings.cursor_enter_polling);
        window.set_scroll_polling(settings.scroll_polling);
        window.set_drag_and_drop_polling(settings.drag_and_drop_polling);
        window.set_maximize_polling(settings.maximize_polling);
        window.set_content_scale_polling(settings.content_scale_polling);
        window.make_current();

        if settings.load_opengl {
            gl::load_with(|symbol| window.get_proc_address(symbol));

            unsafe {
                gl::Disable(gl::CULL_FACE);
                gl::Enable(gl::DEPTH_TEST);
                gl::Enable(gl::MULTISAMPLE);
                gl::Enable(gl::FRAMEBUFFER_SRGB);
            }
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
        &mut self,
        init_extra: T::InitData,
    ) -> Result<(T, Option<T::ExitData>), Error> {
        let mut key_mods = glfw::Modifiers::empty();

        let mut app = T::init(self, init_extra).map_err(Error::App)?;

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

            match app.update(self).map_err(Error::App)? {
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

impl EnvironmentSettings {
    /// Default [`Self::window_dimensions`].
    pub const DEFAULT_WINDOW_DIMENSIONS: (u32, u32) = (1280, 720);

    /// Default [`Self::pos_polling`].
    pub const DEFAULT_POS_POLLING: bool = true;
    /// Default [`Self::size_polling`].
    pub const DEFAULT_SIZE_POLLING: bool = true;
    /// Default [`Self::close_polling`].
    pub const DEFAULT_CLOSE_POLLING: bool = true;
    /// Default [`Self::refresh_polling`].
    pub const DEFAULT_REFRESH_POLLING: bool = true;
    /// Default [`Self::focus_polling`].
    pub const DEFAULT_FOCUS_POLLING: bool = true;
    /// Default [`Self::iconify_polling`].
    pub const DEFAULT_ICONIFY_POLLING: bool = true;
    /// Default [`Self::framebuffer_size_polling`].
    pub const DEFAULT_FRAMEBUFFER_SIZE_POLLING: bool = true;
    /// Default [`Self::key_polling`].
    pub const DEFAULT_KEY_POLLING: bool = true;
    /// Default [`Self::char_polling`].
    pub const DEFAULT_CHAR_POLLING: bool = true;
    /// Default [`Self::char_mods_polling`].
    pub const DEFAULT_CHAR_MODS_POLLING: bool = true;
    /// Default [`Self::mouse_button_polling`].
    pub const DEFAULT_MOUSE_BUTTON_POLLING: bool = true;
    /// Default [`Self::cursor_pos_polling`].
    pub const DEFAULT_CURSOR_POS_POLLING: bool = true;
    /// Default [`Self::cursor_enter_polling`].
    pub const DEFAULT_CURSOR_ENTER_POLLING: bool = true;
    /// Default [`Self::scroll_polling`].
    pub const DEFAULT_SCROLL_POLLING: bool = true;
    /// Default [`Self::drag_and_drop_polling`].
    pub const DEFAULT_DRAG_AND_DROP_POLLING: bool = true;
    /// Default [`Self::maximize_polling`].
    pub const DEFAULT_MAXIMIZE_POLLING: bool = true;
    /// Default [`Self::content_scale_polling`].
    pub const DEFAULT_CONTENT_SCALE_POLLING: bool = true;

    /// Default [`Self::context_version`].
    pub const DEFAULT_CONTEXT_VERSION: (u32, u32) = (4, 5);
    /// Default [`Self::opengl_profile_hint`].
    pub const DEFAULT_OPENGL_PROFILE_HINT: glfw::OpenGlProfileHint = glfw::OpenGlProfileHint::Core;

    /// Default [`Self::load_opengl`].
    pub const DEFAULT_LOAD_OPENGL: bool = true;
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            window_dimensions: Self::DEFAULT_WINDOW_DIMENSIONS,
            pos_polling: Self::DEFAULT_POS_POLLING,
            size_polling: Self::DEFAULT_SIZE_POLLING,
            close_polling: Self::DEFAULT_CLOSE_POLLING,
            refresh_polling: Self::DEFAULT_REFRESH_POLLING,
            focus_polling: Self::DEFAULT_FOCUS_POLLING,
            iconify_polling: Self::DEFAULT_ICONIFY_POLLING,
            framebuffer_size_polling: Self::DEFAULT_FRAMEBUFFER_SIZE_POLLING,
            key_polling: Self::DEFAULT_KEY_POLLING,
            char_polling: Self::DEFAULT_CHAR_POLLING,
            char_mods_polling: Self::DEFAULT_CHAR_MODS_POLLING,
            mouse_button_polling: Self::DEFAULT_MOUSE_BUTTON_POLLING,
            cursor_pos_polling: Self::DEFAULT_CURSOR_POS_POLLING,
            cursor_enter_polling: Self::DEFAULT_CURSOR_ENTER_POLLING,
            scroll_polling: Self::DEFAULT_SCROLL_POLLING,
            drag_and_drop_polling: Self::DEFAULT_DRAG_AND_DROP_POLLING,
            maximize_polling: Self::DEFAULT_MAXIMIZE_POLLING,
            content_scale_polling: Self::DEFAULT_CONTENT_SCALE_POLLING,
            context_version: Self::DEFAULT_CONTEXT_VERSION,
            opengl_profile_hint: Self::DEFAULT_OPENGL_PROFILE_HINT,
            load_opengl: Self::DEFAULT_LOAD_OPENGL,
        }
    }
}

impl EnvironmentSettings {
    /// Set [`Self::window_dimensions`].
    pub fn window_dimensions(mut self, window_dimensions: (u32, u32)) -> Self {
        self.window_dimensions = window_dimensions;
        self
    }

    /// Set [`Self::pos_polling`].
    pub fn pos_polling(mut self, pos_polling: bool) -> Self {
        self.pos_polling = pos_polling;
        self
    }

    /// Set [`Self::size_polling`].
    pub fn size_polling(mut self, size_polling: bool) -> Self {
        self.size_polling = size_polling;
        self
    }

    /// Set [`Self::close_polling`].
    pub fn close_polling(mut self, close_polling: bool) -> Self {
        self.close_polling = close_polling;
        self
    }

    /// Set [`Self::refresh_polling`].
    pub fn refresh_polling(mut self, refresh_polling: bool) -> Self {
        self.refresh_polling = refresh_polling;
        self
    }

    /// Set [`Self::focus_polling`].
    pub fn focus_polling(mut self, focus_polling: bool) -> Self {
        self.focus_polling = focus_polling;
        self
    }

    /// Set [`Self::iconify_polling`].
    pub fn iconify_polling(mut self, iconify_polling: bool) -> Self {
        self.iconify_polling = iconify_polling;
        self
    }

    /// Set [`Self::framebuffer_size_polling`].
    pub fn framebuffer_size_polling(mut self, framebuffer_size_polling: bool) -> Self {
        self.framebuffer_size_polling = framebuffer_size_polling;
        self
    }

    /// Set [`Self::key_polling`].
    pub fn key_polling(mut self, key_polling: bool) -> Self {
        self.key_polling = key_polling;
        self
    }

    /// Set [`Self::char_polling`].
    pub fn char_polling(mut self, char_polling: bool) -> Self {
        self.char_polling = char_polling;
        self
    }

    /// Set [`Self::char_mods_polling`].
    pub fn char_mods_polling(mut self, char_mods_polling: bool) -> Self {
        self.char_mods_polling = char_mods_polling;
        self
    }

    /// Set [`Self::mouse_button_polling`].
    pub fn mouse_button_polling(mut self, mouse_button_polling: bool) -> Self {
        self.mouse_button_polling = mouse_button_polling;
        self
    }

    /// Set [`Self::cursor_pos_polling`].
    pub fn cursor_pos_polling(mut self, cursor_pos_polling: bool) -> Self {
        self.cursor_pos_polling = cursor_pos_polling;
        self
    }

    /// Set [`Self::cursor_enter_polling`].
    pub fn cursor_enter_polling(mut self, cursor_enter_polling: bool) -> Self {
        self.cursor_enter_polling = cursor_enter_polling;
        self
    }

    /// Set [`Self::scroll_polling`].
    pub fn scroll_polling(mut self, scroll_polling: bool) -> Self {
        self.scroll_polling = scroll_polling;
        self
    }

    /// Set [`Self::drag_and_drop_polling`].
    pub fn drag_and_drop_polling(mut self, drag_and_drop_polling: bool) -> Self {
        self.drag_and_drop_polling = drag_and_drop_polling;
        self
    }

    /// Set [`Self::maximize_polling`].
    pub fn maximize_polling(mut self, maximize_polling: bool) -> Self {
        self.maximize_polling = maximize_polling;
        self
    }

    /// Set [`Self::content_scale_polling`].
    pub fn content_scale_polling(mut self, content_scale_polling: bool) -> Self {
        self.content_scale_polling = content_scale_polling;
        self
    }

    /// Set [`Self::context_version`].
    pub fn context_version(mut self, context_version: (u32, u32)) -> Self {
        self.context_version = context_version;
        self
    }

    /// Set [`Self::opengl_profile_hint`].
    pub fn opengl_profile_hint(mut self, opengl_profile_hint: glfw::OpenGlProfileHint) -> Self {
        self.opengl_profile_hint = opengl_profile_hint;
        self
    }

    /// Set [`Self::load_opengl`].
    pub fn load_opengl(mut self, load_opengl: bool) -> Self {
        self.load_opengl = load_opengl;
        self
    }
}
