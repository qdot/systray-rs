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

#[cfg(windows)]
pub mod windows;

pub struct SystrayEvent {
    menu_index: u32,
    menu_checked: bool
}
