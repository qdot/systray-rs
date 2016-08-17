#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
extern crate kernel32;
#[cfg(windows)]
extern crate user32;
#[cfg(windows)]
extern crate shell32;
#[cfg(windows)]
extern crate libc;

#[cfg(windows)]
pub mod windows;
