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

fn prepare_visualization() {
    let vert_shader = Shader::from_vert_source(
        &CString::new(include_str!("shadertoy.vert")).unwrap()
    ).unwrap();

    let frag_shader = Shader::from_frag_source(
        &CString::new(include_str!("flames.frag")).unwrap()
    ).unwrap();

    let shader_program = Program::from_shaders(
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

    let mut vertex_array: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);

        gl::BindFragDataLocation(shader_program.id(), 0, CString::new("fragColor").unwrap().as_ptr());
        
        let pos_attr: gl::types::GLuint = gl::GetAttribLocation(shader_program.id(), CString::new("position").unwrap().as_ptr()) as gl::types::GLuint;
        gl::EnableVertexAttribArray(pos_attr);
        gl::VertexAttribPointer(pos_attr, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint, std::ptr::null());

        let tex_attr: gl::types::GLuint = gl::GetAttribLocation(shader_program.id(), CString::new("vsTex").unwrap().as_ptr()) as gl::types::GLuint;
        gl::EnableVertexAttribArray(tex_attr);
        gl::VertexAttribPointer(tex_attr, 2, gl::FLOAT, gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint, (2 * std::mem::size_of::<f32>()) as *const std::os::raw::c_void);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let time_unif: gl::types::GLint = unsafe { 
        gl::GetUniformLocation(shader_program.id(), CString::new("iTime").unwrap().as_ptr())
    };
    let resolution_unif: gl::types::GLint = unsafe { 
        gl::GetUniformLocation(shader_program.id(), CString::new("iResolution").unwrap().as_ptr())
    };
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

    let time_start = std::time::SystemTime::now();

    canvas.connect_realize(|_| {
        prepare_visualization();
    });

    canvas.connect_render(move |_, _| {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            // let time_elapsed = std::time::SystemTime::now().duration_since(time_start).unwrap();

            // gl::BindVertexArray(vertex_array);
            // gl::Uniform1f(time_unif, time_elapsed.as_secs() as f32 + (time_elapsed.subsec_nanos() / 1_000_000) as f32);
            // gl::Uniform2f(resolution_unif, 150.0, 150.0);
            // gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            // // gl::DrawArrays(
            // //     gl::TRIANGLES, // mode
            // //     0, // starting index in the enabled arrays
            // //     3 // number of indices to be rendered
            // // );
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

