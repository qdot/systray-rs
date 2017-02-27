use std;
use {SystrayError};

pub struct Window {
}

impl Window {
    pub fn new() -> Result<Window, SystrayError> {
        Err(SystrayError::NotImplementedError)
    }
    pub fn quit(&self) {
        unimplemented!()
    }
    pub fn set_tooltip(&self, _: &String) -> Result<(), SystrayError> {
        unimplemented!()
    }
    pub fn add_menu_item<F>(&self, _: &String, _: F) -> Result<u32, SystrayError>
        where F: std::ops::Fn(&Window) -> () + 'static
    {
        unimplemented!()
    }
    pub fn wait_for_message(&mut self) {
        unimplemented!()
    }
    pub fn set_icon_from_buffer(&self, _: &[u8], _: u32, _: u32) -> Result<(), SystrayError> {
        unimplemented!()
    }
}
