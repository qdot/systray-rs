extern crate systray;

fn main() {
    let w = systray::systray::Window::new();
    w.set_tooltip(&"Whatever".to_string());
    w.add_menu_item(&"Something".to_string());
    println!("Waiting on message!");
    w.rx.recv();
}
