extern crate systray;

use systray::windows::{create_window, set_tooltip, add_menu_item, run_loop};

fn main() {
    create_window();
    set_tooltip(&"Whatever".to_string());
    add_menu_item(&"Something".to_string());
    run_loop();
}
