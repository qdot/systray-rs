//! Contains the implementation of the Mac OS X tray icon in the top bar.

use std::{
    self,
    cell::RefCell,
    ffi::c_void,
    mem,
    rc::Rc,
    sync::{
        Mutex,
        mpsc::Sender,
    },
};
use cocoa::{
    appkit::{
        NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps, NSButton, NSImage,
        NSMenu, NSMenuItem, NSRunningApplication, NSStatusBar, NSStatusItem,
        NSSquareStatusItemLength
    },
    base::{id, nil, YES},
    foundation::{NSData, NSSize, NSAutoreleasePool, NSString}
};
use objc::{
    Message,
    declare::ClassDecl,
    runtime::{Class, Object, Sel}
};
use objc_foundation::{INSObject, NSObject};
use objc_id::Id;
use crate::{Application, BoxedError, Callback, Error};

/// The general representation of the Mac OS X application.
pub struct Window {
    /// A reference to systray::Application for callbacks
    systray_application: Option<Rc<RefCell<Box<Application>>>>,
    /// A mutable reference to the `NSApplication` instance of the currently running application.
    application: Mutex<id>,
    /// It seems that we have to use `NSAutoreleasePool` to prevent memory leaks.
    autorelease_pool: Mutex<id>,
    /// `NSMenu` for menu items.
    menu: Mutex<id>,
}

impl Window {
    /// Creates a new instance of the `Window`.
    pub fn new() -> Result<Window, Error> {
        Ok(Window {
            systray_application: None,
            application: unsafe { Mutex::from(NSApp()) },
            autorelease_pool: unsafe { Mutex::from(NSAutoreleasePool::new(nil)) },
            menu: unsafe { Mutex::from(NSMenu::new(nil).autorelease()) },
        })
    }

    /// Sets the systray application
    pub fn set_systray_application(&mut self, application_raw_ptr: *mut Application){
        let application = unsafe { Box::from_raw(application_raw_ptr) };
        self.systray_application = Some(Rc::from(RefCell::from(application)));
    }

    /// Closes the current application.
    pub fn quit(&mut self) {
        if let Ok(application) = self.application.get_mut() {
            unsafe { application.stop_(nil); }
        }
    }

    /// Sets the tooltip (not available for this platform).
    pub fn set_tooltip(&self, _: &str) -> Result<(), Error> {
        Err(Error::OsError("This operating system does not support tooltips for the tray \
                                   items".to_owned()))
    }

    /// Sets the application icon displayed in the tray bar. Accepts a `buffer` to the underlying
    /// image, you can pass even encoded PNG images here. Supports the same list of formats as
    /// `NSImage`.
    pub fn set_icon_from_buffer(&mut self, buffer: &'static [u8], _: u32, _: u32)
        -> Result<(), Error>
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
            return Err(Error::OsError("Could not create `NSData` out of the passed buffer"
                                             .to_owned()));
        }

        let nsimage = unsafe { NSImage::initWithData_(NSImage::alloc(nil), nsdata).autorelease() };
        if nsimage == nil {
            return Err(Error::OsError("Could not create `NSImage` out of the created \
                                             `NSData` buffer".to_owned()));
        }

        unsafe {
            let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);
            let _: () = msg_send![nsimage, setSize:new_size];
            tray_entry.button().setImage_(nsimage);
            if let Ok(menu) = self.menu.get_mut(){
                tray_entry.setMenu_(*menu);
            }
        }

        Ok(())
    }

    /// Starts the application event loop. Calling this function will block the current thread.
    pub fn wait_for_message(&mut self) {
        if let Ok(application) = self.application.get_mut() {
            unsafe {
                application.activateIgnoringOtherApps_(YES);
                NSRunningApplication::currentApplication(nil)
                    .activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
                application.run();
            }
        }
    }

    pub fn set_icon_from_resource(&self, resource_name: &str) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn set_icon_from_file(&mut self, icon_file: &str) -> Result<(), Error> {
        const ICON_WIDTH: f64 = 18.0;
        const ICON_HEIGHT: f64 = 18.0;

        let tray_entry = unsafe {
            NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSSquareStatusItemLength)
                                             .autorelease()
        };

        let path = unsafe {
            NSString::alloc(nil).init_str(icon_file)
        };
        if path == nil {
            return Err(Error::OsError("Could not create `NSString` out of the passed &str"
                                             .to_owned()));
        }

        let nsimage = unsafe { NSImage::initWithContentsOfFile_(NSImage::alloc(nil), path).autorelease() };
        if nsimage == nil {
            return Err(Error::OsError("Could not create `NSImage` out of the created \
                                             `NSData` buffer".to_owned()));
        }

        unsafe {
            let new_size = NSSize::new(ICON_WIDTH, ICON_HEIGHT);
            let _: () = msg_send![nsimage, setSize:new_size];
            tray_entry.button().setImage_(nsimage);
            if let Ok(menu) = self.menu.get_mut(){
                tray_entry.setMenu_(*menu);
            }
        }

        Ok(())
    }

    pub fn add_menu_separator(&mut self, item_idx: u32) -> Result<(), Error> {
        let item = unsafe { 
            NSMenuItem::separatorItem(nil)
        };
        if item == nil {
            return Err(Error::OsError("Could not create `NSMenuItem`."
                                             .to_owned()));
        }

        unsafe {
            if let Ok(menu) = self.menu.get_mut(){
                NSMenu::addItem_(*menu, item);
            }
        }

        Ok(())
    }

    pub fn add_menu_entry(&mut self, item_idx: u32, item_name: &str, callback: Callback) -> Result<(), Error> {
        let blank_key = unsafe { NSString::alloc(nil).init_str("") };
        if blank_key == nil {
            return Err(Error::OsError("Could not create blank `NSString`."
                                             .to_owned()));
        }

        let title = unsafe { NSString::alloc(nil).init_str(item_name) };
        if title == nil {
            return Err(Error::OsError("Could not create `NSString` from the item name."
                                             .to_owned()));
        }

        let action = sel!(call);

        let item = unsafe { 
            NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(title, action, blank_key)
        };
        if item == nil {
            return Err(Error::OsError("Could not create `NSMenuItem`."
                                             .to_owned()));
        }

        unsafe {
            if let Some(app) = &self.systray_application {
                let _ : () = msg_send![item, setTarget: CocoaCallback::from(app.clone(), callback)];
            }
            if let Ok(menu) = self.menu.get_mut(){
                NSMenu::addItem_(*menu, item);
            }
        }

        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }
}

// Devired from https://github.com/rust-sysbar/rust-sysbar/blob/master/src/mac_os/mod.rs
// Copyright (c) 2017 The rs-barfly Developers
// Copyright (c) 2017 The rust-sysbar Developers

pub struct CocoaCallbackState {
    application: Rc<RefCell<Box<Application>>>,
    callback: Callback
}

enum CocoaCallback {}
  
impl CocoaCallback {
    pub fn from(application: Rc<RefCell<Box<Application>>>, callback: Callback) -> Id<Self> {
        let ccs = CocoaCallbackState {
            application,
            callback
        };
        let bccs = Box::new(ccs);

        let ptr = Box::into_raw(bccs);
        let ptr = ptr as *mut c_void as usize;
        let mut oid = <CocoaCallback as INSObject>::new();
        (*oid).setptr(ptr);
        oid
    }

    fn setptr(&mut self, uptr: usize) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut ::objc::runtime::Object);
            obj.set_ivar("_cbptr", uptr);
        }
    }
}

impl CocoaCallbackState {
    pub fn call(&mut self) -> Result<(), BoxedError> {
        if let Ok(mut application) = self.application.try_borrow_mut() {
            return (*self.callback)(&mut application);
        }
        Err(Box::from(Error::OsError("Unable to borrow the application".to_owned())))
    }
}

unsafe impl Message for CocoaCallback {}

impl INSObject for CocoaCallback {
    fn class() -> &'static Class {
        let cname = "CCCallback";

        let mut _class = Class::get(cname);
        if _class.is_none() {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new(&cname, superclass).unwrap();
            decl.add_ivar::<usize>("_cbptr");

            extern "C" fn sysbar_callback_call(obj: &Object, _: Sel) {
                unsafe {
                    let pointer_value: usize = *obj.get_ivar("_cbptr");
                    let callback_pointer = pointer_value as *mut c_void as *mut CocoaCallbackState;
                    let mut boxed_callback: Box<CocoaCallbackState> = Box::from_raw(callback_pointer);
                    {
                       boxed_callback.call();
                    }
                    mem::forget(boxed_callback);
                }
            }

            unsafe {
                decl.add_method(
                    sel!(call),
                    sysbar_callback_call as extern "C" fn(&Object, Sel),
                );
            }

            decl.register();
            _class = Class::get(cname);
        }
        _class.unwrap()
    }
}