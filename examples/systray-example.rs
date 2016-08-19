extern crate systray;
extern crate user32;

fn main() {
    let mut w = systray::systray::Window::new();
    w.set_icon_from_file(&"C:\\Users\\qdot\\code\\git-projects\\systray-rs\\resources\\rust.ico".to_string());
    w.set_tooltip(&"Whatever".to_string());
    w.add_menu_item(&"Something".to_string(), || {
        //w.quit();
    });
    w.add_menu_item(&"Print a thing".to_string(), || {
        println!("Printing a thing!");
    });
    println!("Waiting on message!");
    w.wait_for_message();
}
