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

#[cfg(target_os = "windows")]
#[path="windows.rs"]
pub mod systray;

#[derive(Clone)]
pub enum SystrayError {
    OsError(String),
    UnknownError,
}

pub struct SystrayEvent {
    menu_index: u32,
    menu_checked: bool
}

type Callback = Box<(Fn<(),Output=()> + 'static)>;
