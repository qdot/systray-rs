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
extern crate libc;
#[cfg(target_os = "linux")]
extern crate gtk;
#[cfg(target_os = "linux")]
extern crate glib;
#[cfg(target_os = "linux")]
extern crate libappindicator;

pub mod api;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};

#[derive(Clone, Debug)]
pub enum SystrayError {
    OsError(String),
    NotImplementedError,
    UnknownError,
}

pub struct SystrayEvent {
    menu_index: u32,
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
    window: api::api::Window,
    menu_idx: u32,
    callback: HashMap<u32, Callback>,
    // Each platform-specific window module will set up its own thread for
    // dealing with the OS main loop. Use this channel for receiving events from
    // that thread.
    rx: Receiver<SystrayEvent>,
}

type Callback = Box<(Fn(&mut Application) -> () + 'static)>;

fn make_callback<F>(f: F) -> Callback
    where F: std::ops::Fn(&mut Application) -> () + 'static {
    Box::new(f) as Callback
}

impl Application {
    pub fn new() -> Result<Application, SystrayError> {
        let (event_tx, event_rx) = channel();
        match api::api::Window::new(event_tx) {
            Ok(w) => Ok(Application {
                window: w,
                menu_idx: 0,
                callback: HashMap::new(),
                rx: event_rx
            }),
            Err(e) => Err(e)
        }
    }

    pub fn add_menu_item<F>(&mut self, item_name: &String, f: F) -> Result<u32, SystrayError>
        where F: std::ops::Fn(&mut Application) -> () + 'static {
        let idx = self.menu_idx;
        if let Err(e) = self.window.add_menu_entry(idx, item_name) {
            return Err(e);
        }
        self.callback.insert(idx, make_callback(f));
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn add_menu_separator(&mut self) -> Result<u32, SystrayError> {
        let idx = self.menu_idx;
        if let Err(e) = self.window.add_menu_separator(idx) {
            return Err(e);
        }
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn set_icon_from_file(&self, file: &String) -> Result<(), SystrayError> {
        self.window.set_icon_from_file(file)
    }

    pub fn set_icon_from_resource(&self, resource: &str) -> Result<(), SystrayError> {
        self.window.set_icon_from_resource(resource)
    }

    pub fn shutdown(&self) -> Result<(), SystrayError> {
        self.window.shutdown()
    }

    pub fn set_tooltip(&self, tooltip: &str) -> Result<(), SystrayError> {
        self.window.set_tooltip(tooltip)
    }

    pub fn quit(&self) {
        self.window.quit()
    }

    pub fn wait_for_message(&mut self) {
        loop {
            let msg;
            match self.rx.recv() {
                Ok(m) => msg = m,
                Err(_) => {
                    self.quit();
                    break;
                }
            }
            if self.callback.contains_key(&msg.menu_index) {
                let f = self.callback.remove(&msg.menu_index).unwrap();
                f(self);
                self.callback.insert(msg.menu_index, f);
            }
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
