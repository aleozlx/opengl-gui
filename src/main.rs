extern crate gtk;
extern crate yaml_rust;
extern crate gl;
extern crate epoxy;
extern crate shared_library;
extern crate image;

mod shader_routines;
use yaml_rust::YamlLoader;
use gtk::prelude::*;
use std::ffi::{CString, CStr};
use shared_library::dynamic_library::DynamicLibrary;
use image::GenericImage;
use shader_routines::{Shader, ShaderProgram};

fn append_column(tree: &gtk::TreeView, id: i32) {
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();
    column.pack_start(&cell, true);
    // Association of the view's column with the model's `id` column.
    column.add_attribute(&cell, "text", id);
    column.set_max_width(250);
    column.set_title(match id {
        0 => "Structure",
        1 => "Params",
        _ => ".."
    });
    tree.append_column(&column);
}

fn create_and_fill_model() -> gtk::TreeStore {
    // Creation of a model with two rows.
    let model = gtk::TreeStore::new(&[String::static_type(), u32::static_type()]);

    // Filling up the tree view.
    let vgg16 = &YamlLoader::load_from_str(include_str!("../vgg16-v2.yaml")).unwrap()[0];
    for block in vgg16["nnblocks"].as_vec().unwrap() {
        let blk_name = block["name"].as_str().unwrap();
        let blk_node = model.insert_with_values(None, None, &[0, 1], &[&blk_name, &0]);

        for layer in block["layers"].as_vec().unwrap() {
            let layer_name = layer["class"].as_str().unwrap();
            model.insert_with_values(Some(&blk_node), None, &[0, 1], &[&layer_name, &0]);
        }
    }

    model
}

fn prepare_visualization() -> gl::types::GLuint {
    let vert_shader = Shader::from_vert_source(
        &CString::new(include_str!("canvas.vert")).unwrap()
    ).unwrap();

    let frag_shader = Shader::from_frag_source(
        &CString::new(include_str!("canvas.frag")).unwrap()
    ).unwrap();

    let shader_program = ShaderProgram::from_shaders(
        &[vert_shader, frag_shader]
    ).unwrap();

    shader_program.use_program();
   
    let vertices: Vec<f32> = vec![
    	-1.0, 1.0, 0.0, 0.0, // Top-left
        1.0, 1.0, 1.0, 0.0, // Top-right
        1.0, -1.0, 1.0, 1.0, // Bottom-right
        -1.0, -1.0, 0.0, 1.0  // Bottom-left
    ];
    let mut vertex_buffer: gl::types::GLuint = 0;
    unsafe {
        gl::GenBuffers(1, &mut vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER, // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW, // usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }

    let mut element_buffer: gl::types::GLuint = 0;
    let elements: Vec<u32> = vec![
        0, 1, 2,
		2, 3, 0
    ];
    unsafe {
        gl::GenBuffers(1, &mut element_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (elements.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
            elements.as_ptr() as *const gl::types::GLvoid,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    }

    // let mut vertex_array: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut g_vertex_array);
        gl::BindVertexArray(g_vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

        gl::BindFragDataLocation(shader_program.id(), 0, CString::new("outColor").unwrap().as_ptr());
        
        let pos_attr: gl::types::GLuint = gl::GetAttribLocation(shader_program.id(), CString::new("position").unwrap().as_ptr()) as gl::types::GLuint;
        gl::EnableVertexAttribArray(pos_attr);
        gl::VertexAttribPointer(pos_attr, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint, std::ptr::null());

        let tex_attr: gl::types::GLuint = gl::GetAttribLocation(shader_program.id(), CString::new("vsTex").unwrap().as_ptr()) as gl::types::GLuint;
        gl::EnableVertexAttribArray(tex_attr);
        gl::VertexAttribPointer(tex_attr, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint, (2 * std::mem::size_of::<f32>()) as *const std::os::raw::c_void);

        // let img = image::open("/home/alex/Downloads/avatar64.jpg").unwrap();
        // let img_rgb = img.to_rgb().into_raw();
        // let (width, height) = img.dimensions();

        // let mut texture: gl::types::GLuint = 0;
        // gl::GenTextures(1, &mut texture);
        // gl::BindTexture(gl::TEXTURE_2D, texture);
        // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        // // let inputFormat: gl::types::GLenum = greyscale ? GL_LUMINANCE : GL_BGR;
        // gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        // gl::PixelStorei(gl::UNPACK_ROW_LENGTH, width as i32);
        // gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, img_rgb.as_ptr() as *const std::os::raw::c_void);
    
        // gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        // gl::BindVertexArray(0);

        // gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    return shader_program.id();
}

static mut g_vertex_array: gl::types::GLuint = 0;
static mut time_unif: gl::types::GLint = 0;
static mut resolution_unif: gl::types::GLint = 0;

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

    let canvas: gtk::GLArea = builder.get_object("canvas").unwrap();
    epoxy::load_with(|s| {
        unsafe {
            match DynamicLibrary::open(None).unwrap().symbol(s) {
                Ok(v) => v,
                Err(_) => std::ptr::null(),
            }
        }
    });
    gl::load_with(epoxy::get_proc_addr);

    let time_start = std::time::SystemTime::now();
    
    canvas.connect_realize(|gl_area: &gtk::GLArea| {
        gtk::GLArea::make_current(gl_area);
        let _shaders = prepare_visualization();
        // unsafe {
        //     time_unif = gl::GetUniformLocation(shaders, CString::new("iTime").unwrap().as_ptr());
        //     resolution_unif = gl::GetUniformLocation(shaders, CString::new("iResolution").unwrap().as_ptr());
        // }
    });

    canvas.connect_render(move |_, _| {
        unsafe {
            // gl::Clear(gl::COLOR_BUFFER_BIT);
            // let time_elapsed = std::time::SystemTime::now().duration_since(time_start).unwrap();

            gl::BindVertexArray(g_vertex_array);
            // gl::Uniform1f(time_unif, time_elapsed.as_secs() as f32 + (time_elapsed.subsec_nanos() / 1_000_000) as f32);
            // gl::Uniform2f(resolution_unif, 200.0, 200.0);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            gl::Flush();
        };

        Inhibit(false)
    });

    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}

