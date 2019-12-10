//! Contains the implementation of the Mac OS X tray icon in the top bar.

use std;

use cocoa::appkit::{NSApp, NSApplication, NSButton, NSImage, NSStatusBar, NSStatusItem,
                    NSSquareStatusItemLength};
use cocoa::base::{id, nil};
use cocoa::foundation::{NSData, NSSize, NSAutoreleasePool};

use SystrayError;

/// The generation representation of the Mac OS X application.
pub struct Window {
    /// A mutable reference to the `NSApplication` instance of the currently running application.
    application: id,
    /// It seems that we have to use `NSAutoreleasePool` to prevent memory leaks.
    autorelease_pool: id,
}

impl Window {
    /// Creates a new instance of the `Window`.
    pub fn new() -> Result<Window, SystrayError> {
        Ok(Window {
            application: unsafe { NSApp() },
            autorelease_pool: unsafe { NSAutoreleasePool::new(nil) },
        })
    }

    /// Closes the current application.
    pub fn quit(&self) {
        let _: () = unsafe { msg_send![self.application, terminate] };
    }

    /// Sets the tooltip (not available for this platfor).
    pub fn set_tooltip(&self, _: &String) -> Result<(), SystrayError> {
        Err(SystrayError::OsError("This operating system does not support tooltips for the tray \
                                   items".to_owned()))
    }

    /// Adds an additional item to the tray icon menu.
    pub fn add_menu_item<F>(&self, _: &String, _: F) -> Result<u32, SystrayError>
        where F: std::ops::Fn(&Window) -> () + 'static
    {
        unimplemented!()
    }

    /// Sets the application icon displayed in the tray bar. Accepts a `buffer` to the underlying
    /// image, you can pass even encoded PNG images here. Supports the same list of formats as
    /// `NSImage`.
    pub fn set_icon_from_buffer(&mut self, buffer: &'static [u8], _: u32, _: u32)
        -> Result<(), SystrayError>
    {
        const ICON_WIDTH: f64 = 18.0;
        const ICON_HEIGHT: f64 = 18.0;

        let tray_entry = unsafe {
            NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSSquareStatusItemLength)
                                             .autorelease()
        };

        let nsdata = unsafe {
            NSData::dataWithBytes_length_(nil,
                                          buffer.as_ptr() as *const std::os::raw::c_void,
                                          buffer.len() as u64).autorelease()
        };
        if nsdata == nil {
            return Err(SystrayError::OsError("Could not create `NSData` out of the passed buffer"
                                             .to_owned()));
        }

        let nsimage = unsafe { NSImage::initWithData_(NSImage::alloc(nil), nsdata).autorelease() };
        if nsimage == nil {
            return Err(SystrayError::OsError("Could not create `NSImage` out of the created \
                                             `NSData` buffer".to_owned()));
        }

        unsafe {
            let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);
            let _: () = msg_send![nsimage, setSize:new_size];
            tray_entry.button().setImage_(nsimage);
        }

        Ok(())
    }

    /// Starts the application event loop. Calling this function will block the current thread.
    pub fn wait_for_message(&mut self) {
        unsafe { self.application.run() };
    }
}
