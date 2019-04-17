use actix_files as fs;
use actix_web::{
    App, HttpServer, middleware,
};
use objc::{self, msg_send, sel, sel_impl};

use serde::Deserialize;
use serde_json;

use web_view;

use std::env;

use key::Modifier;

mod menu;
mod key;

fn main() {
    // Serve todomvc example from https://github.com/DenisKolodin/yew/tree/master/examples/todomvc
    std::thread::spawn(|| {
        HttpServer::new(|| {
        let static_dir = env::current_dir().unwrap().join("static");
        println!("static dir: {:#?}", static_dir);
            App::new()
                .service(fs::Files::new("/", static_dir).index_file("index.html"))
        })
            .bind("127.0.0.1:1337").unwrap()
            .workers(1)
            .run()
            .unwrap();
    });

    web_view::builder()
        .size(600, 600)
        .content(web_view::Content::Url("http://127.0.0.1:1337"))
        .invoke_handler(|_, arg| {
            println!("Got arg");
            match serde_json::from_str(arg).unwrap() {
                Event::Load => {
                    add_menu();
                }
            }
            Ok(())
        })
        .user_data(())
        .debug(true)
        .build()
        .unwrap()
        .run()
        .unwrap();
}

fn add_menu() {
    // Initialize event handler in obj-c
    menu::init();

    // Menus need to be created from child to parent

    // "Quit" Menu
    let mut app_menu = menu::Menu::new();
    let app_menu_item = menu::MenuItem::new(
        "Quit",
        Modifier::Command,
        "q",
        || {
            std::process::exit(0);
        },
        1);
    app_menu.add_menu_item(app_menu_item);

    // "Test" Menu
    let app_menu_item2 = menu::MenuItem::new(
        "Test",
        Modifier::Command | Modifier::Shift,
        "t",
        || {
            println!("Just testing")
        },
        2);
    app_menu.add_menu_item(app_menu_item2);

    let mut menubar = menu::Menu::new();
    let mut menubar_item = menu::MenuItem::new(
        "",
        Modifier::None,
        "",
        || {},
        0
    );
    menubar_item.set_submenu(app_menu);
    menubar.add_menu_item(menubar_item);

    // Set menubar to menu manager, so we can receive callbacks
    menu::MENU_MANAGER.lock().unwrap().set_current(menubar);
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Event {
    Load,
}
