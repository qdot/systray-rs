use gtk::{ self, Window as GTKWindow, WindowType, WidgetExt,
           Inhibit, Widget, Menu, MenuShellExt };
use libappindicator::{AppIndicator,
                      AppIndicatorStatus};
use std::cell::{RefCell};
use std::collections::HashMap;
use {SystrayEvent, SystrayError};
use glib;
use std;
use std::thread;
use std::sync::mpsc::{channel, Sender};

// Gtk specific struct that will live only in the Gtk thread, since a lot of the
// base types involved don't implement Send (for good reason).
pub struct GtkSystrayApp {
    menu: gtk::Menu,
    ai: AppIndicator,
    menu_items: RefCell<HashMap<u32, gtk::MenuItem>>,
    event_tx: Sender<SystrayEvent>
}

thread_local!(static GTK_STASH: RefCell<Option<GtkSystrayApp>> = RefCell::new(None));

pub struct MenuItemInfo {
    mid: u32,
    title: String,
    tooltip: String,
    disabled: bool,
    checked: bool
}

impl GtkSystrayApp {
    pub fn new(event_tx: Sender<SystrayEvent>) -> Result<GtkSystrayApp, SystrayError> {
        if let Err(e) = gtk::init() {
            return Err(SystrayError::OsError(format!("{}", "Gtk init error!")));
        }
        let mut m = gtk::Menu::new();
        let mut ai = AppIndicator::new("", "");
        ai.set_status(AppIndicatorStatus::APP_INDICATOR_STATUS_ACTIVE);
        ai.set_menu(&mut m);
        Ok(GtkSystrayApp {
            menu: m,
            ai: ai,
            menu_items: RefCell::new(HashMap::new()),
            event_tx: event_tx
        })
    }

    pub fn add_menu_entry(&self, item_name: &String) {
        // MenuItemInfo *mii = (MenuItemInfo*)data;
	      // GList* it;
	      // for(it = global_menu_items; it != NULL; it = it->next) {
		    //     MenuItemNode* item = (MenuItemNode*)(it->data);
		    //     if(item->menu_id == mii->menu_id){
			  //         gtk_menu_item_set_label(GTK_MENU_ITEM(item->menu_item), mii->title);
			  //         break;
		    //     }
	      // }
        let m = gtk::MenuItem::new_with_label(item_name);
        self.menu.append(&m);
        self.menu.show_all();
        //self.menu_items.insert(self.menu_idx, m);
	      // // menu id doesn't exist, add new item
	      // if(it == NULL) {
		    //     GtkWidget *menu_item = gtk_menu_item_new_with_label(mii->title);
		    //     int *id = malloc(sizeof(int));
		    //     *id = mii->menu_id;
		    //     g_signal_connect_swapped(G_OBJECT(menu_item), "activate", G_CALLBACK(_systray_menu_item_selected), id);
		    //     gtk_menu_shell_append(GTK_MENU_SHELL(global_tray_menu), menu_item);

		    //     MenuItemNode* new_item = malloc(sizeof(MenuItemNode));
		    //     new_item->menu_id = mii->menu_id;
		    //     new_item->menu_item = menu_item;
		    //     GList* new_node = malloc(sizeof(GList));
		    //     new_node->data = new_item;
		    //     new_node->next = global_menu_items;
		    //     if(global_menu_items != NULL) {
			  //         global_menu_items->prev = new_node;
		    //     }
		    //     global_menu_items = new_node;
		    //     it = new_node;
	      // }
	      // GtkWidget * menu_item = GTK_WIDGET(((MenuItemNode*)(it->data))->menu_item);
	      // gtk_widget_set_sensitive(menu_item, mii->disabled == 1 ? FALSE : TRUE);
	      // gtk_widget_show_all(global_tray_menu);

	      // free(mii->title);
	      // free(mii->tooltip);
	      // free(mii);
        // return FALSE;
    }
}

pub struct Window {
    gtk_loop: Option<thread::JoinHandle<()>>
}

type Callback = Box<(Fn(&GtkSystrayApp) -> () + 'static)>;

// Convenience function to clean up thread local unwrapping
fn run_on_gtk_thread<F>(f: F)
    where F: std::ops::Fn(&GtkSystrayApp) -> () + Send + 'static {
    // Note this is glib, not gtk. Calling gtk::idle_add will panic us due to
    // being on different threads. glib::idle_add can run across threads.
    glib::idle_add(move || {
        GTK_STASH.with(|stash| {
            let stash = stash.borrow();
            let stash = stash.as_ref();
            if let Some(stash) = stash {
                f(stash);
            }
        });
        gtk::Continue(false)
    });
}

impl Window {
    pub fn new(event_tx: Sender<SystrayEvent>) -> Result<Window, SystrayError> {
        let (tx, rx) = channel();
        let gtk_loop = thread::spawn(move || {
            GTK_STASH.with(|stash| {
                match GtkSystrayApp::new(event_tx) {
                    Ok(data) => {
                        (*stash.borrow_mut()) = Some(data);
                        tx.send(Ok(()));
                    }
                    Err(e) => {
                        tx.send(Err(e));
                        return;
                    }
                }
            });
            gtk::main();
        });
        match rx.recv().unwrap() {
            Ok(()) => Ok(Window {
                gtk_loop: Some(gtk_loop)
            }),
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn add_menu_entry(&self, item_name: &String) -> Result<u32, SystrayError> {
        let n = item_name.clone();
        run_on_gtk_thread(move |stash : &GtkSystrayApp| {
            stash.add_menu_entry(&n);
        });
        Ok(0)
    }

    pub fn add_menu_seperator(&self) -> Result<u32, SystrayError> {
        panic!("Not implemented on this platform!");
    }

    pub fn set_icon_from_file(&self, file: &str) -> Result<(), SystrayError> {
        panic!("Not implemented on this platform!");
    }

    pub fn set_icon_from_resource(&self, resource: &str) -> Result<(), SystrayError> {
        panic!("Not implemented on this platform!");
    }

    pub fn shutdown(&self) -> Result<(), SystrayError> {
        Ok(())
    }

    pub fn set_tooltip(&self, tooltip: &str) -> Result<(), SystrayError> {
        panic!("Not implemented on this platform!");
    }

    pub fn quit(&self) {
        glib::idle_add(|| {
            gtk::main_quit();
            glib::Continue(false)
        });
    }

}
