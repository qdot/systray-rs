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
    menu_idx: Cell<u32>,
    callback: RefCell<HashMap<u32, Callback>>,
    // Each platform-specific window module will set up its own thread for
    // dealing with the OS main loop. Use this channel for receiving events from
    // that thread.
    rx: Receiver<SystrayEvent>,
}

type Callback = Box<(Fn(&Application) -> () + 'static)>;

fn make_callback<F>(f: F) -> Callback
    where F: std::ops::Fn(&Application) -> () + 'static {
    Box::new(f) as Callback
}

impl Application {
    pub fn new() -> Result<Application, SystrayError> {
        let (event_tx, event_rx) = channel();
        match api::api::Window::new(event_tx) {
            Ok(w) => Ok(Application {
                window: w,
                menu_idx: Cell::new(0),
                callback: RefCell::new(HashMap::new()),
                rx: event_rx
            }),
            Err(e) => Err(e)
        }
    }

    pub fn add_menu_item<F>(&self, item_name: &String, f: F) -> Result<u32, SystrayError>
        where F: std::ops::Fn(&Application) -> () + 'static {
        let idx = match self.window.add_menu_entry(item_name) {
            Ok(i) => i,
            Err(e) => {
                return Err(e);
            }
        };
        let mut m = self.callback.borrow_mut();
        m.insert(idx, make_callback(f));
        Ok(idx)
    }

    pub fn add_menu_seperator(&self) -> Result<u32, SystrayError> {
        self.window.add_menu_seperator()
    }

    pub fn set_icon_from_file(&self, file: &str) -> Result<(), SystrayError> {
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
            if (*self.callback.borrow()).contains_key(&msg.menu_index) {
                let f = (*self.callback.borrow_mut()).remove(&msg.menu_index).unwrap();
                f(&self);
                (*self.callback.borrow_mut()).insert(msg.menu_index, f);
            }
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
