extern crate gtk;
extern crate yaml_rust;
use yaml_rust::YamlLoader;
use gtk::prelude::*;


fn append_column(tree: &gtk::TreeView, id: i32) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();
    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    column.set_max_width(200);
    tree.append_column(&column);
}

fn create_and_fill_model() -> gtk::TreeStore {
    // Creation of a model with two rows.
    let model = gtk::TreeStore::new(&[String::static_type(), u32::static_type()]);

    // Filling up the tree view.
    let vgg16 = &YamlLoader::load_from_str(include_str!("../vgg16.yaml")).unwrap()[0];
    for block in vgg16["nnblocks"].as_vec().unwrap() {
        let blk_name = block["name"].as_str().unwrap();
        let blk_node = model.insert_with_values(None, None, &[0, 1], &[&blk_name, &0]);

        for layer in block["layers"].as_vec().unwrap() {
            let layer_name = layer.as_str().unwrap();
            model.insert_with_values(Some(&blk_node), None, &[0, 1], &[&layer_name, &0]);
        }
    }

    model
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("ui-design.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let window: gtk::Window = builder.get_object("main_window").unwrap();
    window.set_title("UI Design");
    
    let tree_nnblocks: gtk::TreeView = builder.get_object("tree_nnblocks").unwrap();
    append_column(&tree_nnblocks, 0);
    append_column(&tree_nnblocks, 1);
    let model = create_and_fill_model();
    // Setting the model into the view.
    tree_nnblocks.set_model(Some(&model));
    tree_nnblocks.expand_all();
    // // ref: http://gtk-rs.org/docs/gtk/
    // let image = Image::new_from_file("/home/alex/3cv-blog/static/icons/favicon-32x32.png");
    // window.add(&image);
    window.show_all();
    gtk::main();
}

