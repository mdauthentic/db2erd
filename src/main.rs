#![allow(non_snake_case)]
mod app;
mod parser;
use app::App;
use dioxus_desktop::{
    tao::menu::{MenuBar, MenuItem},
    LogicalSize, WindowBuilder,
};

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_window(
                WindowBuilder::new()
                    .with_title("DB 2 ERD")
                    .with_inner_size(LogicalSize::new(1280, 900))
                    .with_menu({
                        let mut menu = MenuBar::new();
                        let mut app_menu = MenuBar::new();
                        app_menu.add_native_item(MenuItem::Minimize);
                        app_menu.add_native_item(MenuItem::Quit);

                        menu.add_submenu("&Dioxus Demo", true, app_menu);
                        menu
                    }),
            )
            .with_custom_head(r#"<link rel="stylesheet" href="public/tailwind.css">"#.to_string()),
    );
}
