// Needed for msg_send! macro
#[cfg(any(target_os = "macos"))]
#[macro_use]
extern crate objc;

#[cfg(any(target_os = "macos"))]
extern crate crossbeam;

// Systray Lib
pub mod api;

use std::{
    collections::HashMap,
    error, fmt, thread,
    sync::{
        Arc,
        Mutex,
        MutexGuard,
        mpsc::{channel, Receiver},
    }
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
    window: Arc<Mutex<api::api::Window>>,
    menu_idx: u32,
    callback: HashMap<u32, Callback>,
    // Each platform-specific window module will set up its own thread for
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
    pub fn new() -> Result<Application, Error> {
        let (event_tx, event_rx) = channel();
        match api::api::Window::new(event_tx) {
            Ok(w) => Ok(Application {
                window: Arc::from(Mutex::from(w)),
                menu_idx: 0,
                callback: HashMap::new(),
                rx: event_rx,
            }),
            Err(e) => Err(e),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Application, Error> {
        let (event_tx, event_rx) = channel();
        match api::api::Window::new(Mutex::from(event_tx)) {
            Ok(w) => Ok(Application {
                window: Arc::from(Mutex::from(w)),
                menu_idx: 0,
                callback: HashMap::new(),
                rx: event_rx,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn get_window<'a>(&'a mut self) -> Result<MutexGuard<'a, api::api::Window>, Error> {
        let arc = Box::leak(Box::from(self.window.clone()));
        match arc.lock() {
            Ok(w) => Ok(w),
            Err(_) => Err(Error::OsError("Error acquiring lock for window".to_owned()))
        }
    }

    pub fn add_menu_item<F, E>(&mut self, item_name: &str, f: F) -> Result<u32, Error>
    where
        F: FnMut(&mut Application) -> Result<(), E> + Send + Sync + 'static,
        E: error::Error + Send + Sync + 'static,
    {
        let idx = self.menu_idx;
        if let Err(e) = self.get_window()?.add_menu_entry(idx, item_name) {
            return Err(e);
        }
        self.callback.insert(idx, make_callback(f));
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn add_menu_separator(&mut self) -> Result<u32, Error> {
        let idx = self.menu_idx;
        if let Err(e) = self.get_window()?.add_menu_separator(idx) {
            return Err(e);
        }
        self.menu_idx += 1;
        Ok(idx)
    }

    pub fn set_icon_from_file(&mut self, file: &str) -> Result<(), Error> {
        self.get_window()?.set_icon_from_file(file)
    }

    pub fn set_icon_from_resource(&mut self, resource: &str) -> Result<(), Error> {
        self.get_window()?.set_icon_from_resource(resource)
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub fn set_icon_from_buffer(
        &mut self,
        buffer: &'static [u8],
        width: u32,
        height: u32,
    ) -> Result<(), Error> {
        self.get_window()?.set_icon_from_buffer(buffer, width, height)
    }

    pub fn shutdown(&mut self) -> Result<(), Error> {
        self.get_window()?.shutdown()
    }

    pub fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error> {
        self.get_window()?.set_tooltip(tooltip)
    }

    pub fn quit(&mut self) -> Result<(), Error> {
        self.get_window()?.quit();
        Ok(())
    }

    fn run_event_loop(&mut self) -> Result<(), Error> {
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
    
    #[cfg(not(target_os = "macos"))]
    pub fn wait_for_message(&mut self) -> Result<(), Error> {
        self.run_event_loop()
    }

    #[cfg(target_os = "macos")]
    pub fn wait_for_message<'a>(&'a mut self) -> Result<(), Error> {
        crossbeam::scope(|scope| {
            let thread_window_mutex = self.window.clone();
            let evloop_thread = scope.spawn(move || {
                self.run_event_loop();
            });
            let mut thread_window = thread_window_mutex.lock().unwrap();
            thread_window.wait_for_message();
            evloop_thread.join();
        });
        Ok(())
    }

}

impl Drop for Application {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}
