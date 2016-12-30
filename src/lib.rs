// Systray Lib

#[macro_use]
extern crate log;
#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
extern crate kernel32;
#[cfg(target_os = "windows")]
extern crate user32;
#[cfg(target_os = "windows")]
extern crate shell32;
#[cfg(target_os = "windows")]
extern crate libc;

pub mod api;

#[derive(Clone,Debug)]
pub enum SystrayError {
    OsError(String),
    NotImplementedError,
    UnknownError,
}

pub struct SystrayEvent {
    menu_index: u32,
    menu_checked: bool
}

impl std::fmt::Display for SystrayError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            &SystrayError::OsError(ref err_str) => write!(f, "OsError: {}", err_str),
            &SystrayError::NotImplementedError => write!(f, "Functionality is not implemented yet"),
            &SystrayError::UnknownError => write!(f, "Unknown error occurrred"),
        }
    }
}

pub struct Application {
    pub window: api::api::Window
}

impl Application {
    pub fn new() -> Result<Application, SystrayError> {
        match api::api::Window::new() {
            Ok(w) => Ok(Application {
                window: w
            }),
            Err(e) => Err(e)
        }
    }
}

type Callback = Box<(Fn(&api::api::Window) -> () + 'static)>;

fn make_callback<F>(f: F) -> Callback
    where F: std::ops::Fn(&api::api::Window) -> () + 'static {
    Box::new(f) as Callback
}
