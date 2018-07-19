extern crate gtk;
extern crate yaml_rust;
extern crate gl;
extern crate epoxy;
extern crate shared_library;
use yaml_rust::YamlLoader;
use gtk::prelude::*;
use std::ffi::{CString, CStr};
use shared_library::dynamic_library::DynamicLibrary;

struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<Shader, String> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }

    fn from_vert_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::VERTEX_SHADER)
    }

    fn from_frag_source(source: &CStr) -> Result<Shader, String> {
        Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn create_empty_cstring(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    buffer.extend([b' '].iter().cycle().take(len)); // fill it with len spaces
    unsafe { CString::from_vec_unchecked(buffer) } // convert buffer to CString
}

fn shader_from_source(source: &CStr, kind: gl::types::GLenum) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }
    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }
    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error = create_empty_cstring(len as usize);
        unsafe {
            gl::GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar
            );
        }
        Err(error.to_string_lossy().into_owned())
    }
    else{
        Ok(id)
    }
}

struct Program {
    id: gl::types::GLuint,
}

impl Program {
    pub fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe { gl::AttachShader(program_id, shader.id()); }
        }

        unsafe { gl::LinkProgram(program_id); }
        
        let mut success: gl::types::GLint = 1;
        unsafe {
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_empty_cstring(len as usize);

            unsafe {
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe { gl::DetachShader(program_id, shader.id()); }
        }

        Ok(Program { id: program_id })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

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

    canvas.connect_render(|_, _| {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::Flush();
        };

        Inhibit(false)
    });
    // glib::signal::connect(&canvas, "render", G_CALLBACK (render), None);

    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
}

