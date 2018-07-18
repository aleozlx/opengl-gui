extern crate gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, TreeView};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("ui-design.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let window: gtk::Window = builder.get_object("main_window").unwrap();
    window.set_title("UI Design");
    window.set_default_size(350, 70);

    let nn_layers: gtk::TreeView = builder.get_object("nn_layers").unwrap();

    // // ref: http://gtk-rs.org/docs/gtk/
    // let image = Image::new_from_file("/home/alex/3cv-blog/static/icons/favicon-32x32.png");
    // window.add(&image);
    window.show_all();

    gtk::main();
}

