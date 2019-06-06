#![windows_subsystem = "windows"]

extern crate systray;

//#[cfg(target_os = "windows")]
fn main() {
    let mut app;
    match systray::Application::new() {
        Ok(w) => app = w,
        Err(_) => panic!("Can't create window!"),
    }
    // w.set_icon_from_file(&"C:\\Users\\qdot\\code\\git-projects\\systray-rs\\resources\\rust.ico".to_string());
    // w.set_tooltip(&"Whatever".to_string());
    app.set_icon_from_file("/usr/share/gxkb/flags/ua.png")
        .ok();
    app.add_menu_item("Print a thing", |_| {
        println!("Printing a thing!");
    })
    .ok();
    app.add_menu_item("Add Menu Item", |window| {
        window
            .add_menu_item("Interior item", |_| {
                println!("what");
            })
            .ok();
        window.add_menu_separator().ok();
    })
    .ok();
    app.add_menu_separator().ok();
    app.add_menu_item("Quit", |window| {
        window.quit();
    })
    .ok();
    println!("Waiting on message!");
    app.wait_for_message();
}

// #[cfg(not(target_os = "windows"))]
// fn main() {
//     panic!("Not implemented on this platform!");
// }
