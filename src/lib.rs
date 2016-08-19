// Systray Lib

#![feature(unboxed_closures)]
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

#[derive(Clone)]
pub enum SystrayError {
    OsError(String),
    UnknownError,
}

pub struct SystrayEvent {
    menu_index: u32,
    menu_checked: bool
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

type Callback = Box<(Fn<(),Output=()> + 'static)>;

fn make_callback<F>(f: F) -> Callback
    where F: std::ops::Fn<(),Output=()> + 'static {
    Box::new(f) as Callback
}
