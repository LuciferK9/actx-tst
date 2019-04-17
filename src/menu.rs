use std::sync::Mutex;

use cocoa::appkit::{NSApp, NSApplication, NSMenu, NSMenuItem};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use lazy_static::lazy_static;
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::{ClassDecl, MethodImplementation};
use objc::runtime::{Class, Object, Sel};

use crate::key::Modifier;

lazy_static! {
    pub static ref MENU_MANAGER: Mutex<MenuManager> = Mutex::new(MenuManager{
        current: None,
    });
}

pub struct Menu {
    raw: id,
    items: Vec<Box<MenuItem>>,
}

pub struct MenuItem {
    raw: id,
    callback: Box<Fn()>,
    submenu: Option<Box<Menu>>,
    id: i32,
}

pub struct MenuManager {
    current: Option<Box<Menu>>,
}

unsafe impl Sync for MenuManager {}

unsafe impl Send for MenuManager {}

impl Menu {
    pub fn new() -> Box<Self> {
        let raw_menu: id;
        unsafe {
            raw_menu = NSMenu::alloc(nil).autorelease();
        }
        Box::new(Self { raw: raw_menu, items: vec![] })
    }
    pub fn add_menu_item(&mut self, menu_item: Box<MenuItem>) {
        unsafe {
            self.raw.addItem_(&mut (*menu_item.raw));
        }
        self.items.push(menu_item);
    }

    pub fn get_from_tag(&self, tag: i32) -> Option<&MenuItem> {
        for item in &self.items {
            if let Some(item) = item.get_from_tag(tag) {
                return Some(item);
            }
        }
        None
    }
}

impl MenuItem {
    pub fn new(title: &str, modifier: Modifier, shortcut: &str, callback: impl Fn() + 'static, index: i32) -> Box<Self> {
        let callback = Box::new(callback);
        let raw_menu_item: id;
        unsafe {
            let action = selector("dispatchEvent:");
            let key = NSString::alloc(nil).init_str(shortcut);
            let title = NSString::alloc(nil).init_str(title);
            raw_menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                title,
                action,
                key,
            ).autorelease();
            if modifier != Modifier::None {
                let _: () = msg_send![raw_menu_item, setKeyEquivalentModifierMask:modifier];
            }
            let _: () = msg_send![raw_menu_item, setTarget:class![Menu]];
            let _: () = msg_send![raw_menu_item, setTag:index];
        }
        Box::new(Self {
            raw: raw_menu_item,
            callback,
            id: index,
            submenu: None,
        })
    }

    pub fn set_submenu(&mut self, submenu: Box<Menu>) {
        let subraw = submenu.raw.clone();
        self.submenu = Some(submenu);
        unsafe {
            self.raw.setSubmenu_(&mut (*subraw));
        }
    }

    pub fn get_from_tag(&self, tag: i32) -> Option<&MenuItem> {
        if self.id == tag {
            return Some(&self);
        }
        let item = match &self.submenu {
            Some(submenu) => {
                submenu.get_from_tag(tag)
            }
            None => None
        };
        item
    }
}

impl MenuManager {
    pub fn set_current(&mut self, menu: Box<Menu>) {
        set_main_menu(&menu);
        self.current = Some(menu);
    }
    pub fn get_from_tag(&self, tag: i32) -> Option<&MenuItem> {
        match &self.current {
            Some(menu) => {
                return menu.get_from_tag(tag);
            }
            None => return None,
        }
    }
}

pub fn init() {
    let superclass = class!(NSObject);
    let mut event_class = ClassDecl::new("Menu", superclass).unwrap();

    unsafe {
        event_class.add_class_method(sel!(dispatchEvent:), dispatch_event as extern fn(&Class, Sel, id));
    }
    event_class.register();
}

pub fn set_main_menu(menu: &Box<Menu>) {
    unsafe {
        let app = NSApp();
        app.setMainMenu_(&mut *menu.raw);
        msg_send![&mut *menu.raw, update];
    }
}

extern fn dispatch_event(this: &Class, _cmd: Sel, caller: id) {
    let tag: i32;
    unsafe {
        tag = msg_send![caller, tag];
        let manager = MENU_MANAGER.lock().unwrap();
        let item = manager.get_from_tag(tag);
        match item {
            Some(item) => { (item.callback)() }
            _ => {}
        }
    }
}


