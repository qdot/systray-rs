//! Contains the implementation of the Mac OS X tray icon in the top bar.

use std::{
    self,
    collections::HashMap,
    ffi::c_void,
    mem,
    sync::{
        Arc,
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
use crate::{Error, SystrayEvent};

/// The generation representation of the Mac OS X application.
pub struct Window {
    /// A mutable reference to the `NSApplication` instance of the currently running application.
    application: Mutex<id>,
    /// It seems that we have to use `NSAutoreleasePool` to prevent memory leaks.
    autorelease_pool: Mutex<id>,
    /// `NSMenu` for menu items.
    menu: Mutex<id>,
    /// Sender for menu events.
    event_tx: Arc<Mutex<Sender<SystrayEvent>>>,
}

impl Window {
    /// Creates a new instance of the `Window`.
    pub fn new(event_tx: Mutex<Sender<SystrayEvent>>) -> Result<Window, Error> {
        Ok(Window {
            application: unsafe { Mutex::from(NSApp()) },
            autorelease_pool: unsafe { Mutex::from(NSAutoreleasePool::new(nil)) },
            menu: unsafe { Mutex::from(NSMenu::new(nil).autorelease()) },
            event_tx: Arc::from(Mutex::from(event_tx)),
        })
    }

    /// Closes the current application.
    pub fn quit(&mut self) {
        if let Ok(application) = self.application.get_mut() {
            let _: () = unsafe { msg_send![*application, terminate] };
        }
    }

    /// Sets the tooltip (not available for this platform).
    pub fn set_tooltip(&self, _: &str) -> Result<(), Error> {
        Err(Error::OsError("This operating system does not support tooltips for the tray \
                                   items".to_owned()))
    }

    /// Adds an additional item to the tray icon menu.
    pub fn add_menu_item<F>(&self, _: &String, _: F) -> Result<u32, Error>
        where F: std::ops::Fn(&Window) -> () + 'static
    {
        unimplemented!()
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

    pub fn set_icon_from_file(&self, icon_file: &str) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn add_menu_separator(&self, item_idx: u32) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn add_menu_entry(&mut self, item_idx: u32, item_name: &str) -> Result<(), Error> {
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

        let target = CocoaCallback::from(self.event_tx.clone(), item_idx);

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
            let _ : () = msg_send![item, setTarget: target];
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

unsafe impl Send for Window {}

pub struct CocoaCallbackState {
    event_tx: Arc<Mutex<Sender<SystrayEvent>>>,
    idx: u32,
}

enum CocoaCallback {}
  
impl CocoaCallback {
    pub fn from(event_tx: Arc<Mutex<Sender<SystrayEvent>>>, idx: u32) -> Id<Self> {
        let ccs = CocoaCallbackState {
            event_tx: event_tx,
            idx: idx
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
    pub fn call(&mut self){
        if let Ok(event_tx) = self.event_tx.lock() {
            event_tx.send(SystrayEvent {
                menu_index: self.idx
            });
        }
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