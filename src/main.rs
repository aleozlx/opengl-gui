extern crate gtk;
use gtk::prelude::*;
use gtk::{Button, Window, WindowType, Image};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("First GTK+ Program");
    window.set_default_size(350, 70);
    let button = Button::new_with_label("Click me!");
    // window.add(&button);
    // ref: http://gtk-rs.org/docs/gtk/
    let image = Image::new_from_file("/home/alex/3cv-blog/static/icons/favicon-32x32.png");
    window.add(&image);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(|_| {
        println!("Clicked!");
    });

    gtk::main();
}

