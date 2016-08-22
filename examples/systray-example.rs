extern crate systray;

#[cfg(target_os = "windows")]
fn main() {
    let mut app;
    match systray::Application::new() {
        Ok(w) => app = w,
        Err(e) => panic!("Can't create window!")
    }
    let mut w = &mut app.window;
    w.set_icon_from_file(&"C:\\Users\\qdot\\code\\git-projects\\systray-rs\\resources\\rust.ico".to_string());
    w.set_tooltip(&"Whatever".to_string());
    w.add_menu_item(&"Print a thing".to_string(), |window| {
        println!("Printing a thing!");
    });
    w.add_menu_item(&"Add Menu Item".to_string(), |window| {
        window.add_menu_item(&"Interior item".to_string(), |window| {
            println!("what");
        });
        window.add_menu_separator();
    });
    w.add_menu_separator();
    w.add_menu_item(&"Quit".to_string(), |window| {
        window.quit();
    });
    println!("Waiting on message!");
    w.wait_for_message();
}

#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("Not implemented on this platform!");
}
