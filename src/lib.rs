// Needed for msg_send! macro
#[cfg(any(target_os = "macos"))]
#[macro_use]
extern crate objc;

// Systray Lib
pub mod api;

use std::{
    cell::RefCell,
    collections::HashMap,
    error, fmt,
    rc::Rc,
    sync::mpsc::{channel, Receiver},
};

type BoxedError = Box<dyn error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    OsError(String),
    NotImplementedError,
    UnknownError,
    Error(BoxedError),
}

impl From<BoxedError> for Error {
    fn from(value: BoxedError) -> Self {
        Error::Error(value)
    }
}

pub struct SystrayEvent {
    menu_index: u32,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use self::Error::*;

        match *self {
            OsError(ref err_str) => write!(f, "OsError: {}", err_str),
            NotImplementedError => write!(f, "Functionality is not implemented yet"),
            UnknownError => write!(f, "Unknown error occurrred"),
            Error(ref e) => write!(f, "Error: {}", e),
        }
    }
}

pub struct Application {
    window: Rc<RefCell<Box<api::api::Window>>>,
    #[cfg(target_os = "macos")]
    window_raw_ptr: *mut api::api::Window,
    menu_idx: u32,
    callback: HashMap<u32, Callback>,
    // Each non-macOS platform-specific window module will set up its own thread for
    // dealing with the OS main loop. Use this channel for receiving events from
    // that thread.
    rx: Receiver<SystrayEvent>,
}

type Callback =
    Box<(dyn FnMut(&mut Application) -> Result<(), BoxedError> + Send + Sync + 'static)>;

fn make_callback<F, E>(mut f: F) -> Callback
where
    F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
    E: error::Error + Send + Sync + 'static,
{
    Box::new(move |a: &mut Application| match f(a) {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e) as BoxedError),
    }) as Callback
}

impl Application {
    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Result<Box<Application>, Error> {
        let (event_tx, event_rx) = channel();
        match api::api::Window::new(event_tx) {
            Ok(w) => Ok(Box::from(Application {
                window: Rc::from(RefCell::from(Box::from(w))),
                menu_idx: 0,
                callback: HashMap::new(),
                rx: event_rx,
            })),
            Err(e) => Err(e),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Box<Application>, Error> {
        let (event_tx, event_rx) = channel();
        
        match api::api::Window::new() {
            Ok(w) => {
                let window_raw_ptr = Box::into_raw(Box::from(w));
                let window = unsafe { Box::from_raw(window_raw_ptr) };
                let window_rc = Rc::from(RefCell::from(window));
                let application_raw_ptr = Box::into_raw(Box::from(Application {
                    window: window_rc.clone(),
                    window_raw_ptr,
                    menu_idx: 0,
                    callback: HashMap::new(),
                    rx: event_rx,
                }));

                let mut application_window = window_rc.borrow_mut();
                application_window.set_systray_application(application_raw_ptr);

                let application = unsafe { Box::from_raw(application_raw_ptr) };
                Ok(application)
            },
            Err(e) => Err(e),
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn add_menu_item<F, E>(&mut self, item_name: &str, f: F) -> Result<u32, Error>
    where
        F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
        E: error::Error + Send + Sync + 'static,
    {
        let idx = self.menu_idx;
        if let Err(e) = self.window.try_borrow_mut()?.add_menu_entry(idx, item_name) {
            return Err(e);
        }
        self.callback.insert(idx, make_callback(f));
        self.menu_idx += 1;
        Ok(idx)
    }

    #[cfg(target_os = "macos")]
    pub fn add_menu_item<F, E>(&mut self, item_name: &str, f: F) -> Result<u32, Error>
    where
        F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
        E: error::Error + Send + Sync + 'static,
    {
        let idx = self.menu_idx;
        if let Err(e) = self.window.try_borrow_mut()?.add_menu_entry(idx, item_name, make_callback(f)) {
            return Err(e);
        }
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn add_menu_separator(&mut self) -> Result<u32, Error> {
        let idx = self.menu_idx;
        if let Err(e) = self.window.try_borrow_mut()?.add_menu_separator(idx) {
            return Err(e);
        }
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn set_icon_from_file(&mut self, file: &str) -> Result<(), Error> {
        self.window.try_borrow_mut()?.set_icon_from_file(file)
    }

    pub fn set_icon_from_resource(&mut self, resource: &str) -> Result<(), Error> {
        self.window.try_borrow_mut()?.set_icon_from_resource(resource)
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn set_icon_from_buffer(
        &mut self,
        buffer: &'static [u8],
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        self.window.try_borrow_mut()?.set_icon_from_buffer(buffer, width, height)
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.window.try_borrow_mut()?.shutdown()
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        self.window.try_borrow_mut()?.set_tooltip(tooltip)
    }

    pub fn quit(&mut self) -> Result<(), Error> {
        self.window.try_borrow_mut()?.quit();
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    fn wait_for_message(&mut self) -> Result<(), Error> {
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
                if let Some(mut f) = self.callback.remove(&msg.menu_index) {
                    f(self)?;
                    self.callback.insert(msg.menu_index, f);
                }
            }
        }

        Ok(())
    }
    
    #[cfg(target_os = "macos")]
    pub fn wait_for_message<'a>(&'a mut self) -> Result<(), Error> {
        let mut window = unsafe { Box::from_raw(self.window_raw_ptr) };
        window.wait_for_message();
        
        Ok(())
    }

}

impl Drop for Application {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}

impl std::convert::From<std::cell::BorrowMutError> for Error {
    fn from(err: std::cell::BorrowMutError) -> Self {
        Error::OsError(format!("{}", err))
    }
}