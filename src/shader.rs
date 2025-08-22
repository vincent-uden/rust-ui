use std::{ffi::CString, fs, path::Path};

use anyhow::{Result, anyhow};
use tracing::error;

pub trait UniformValue {
    fn set_uniform(location: gl::types::GLint, value: &Self);
}

impl UniformValue for f32 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::Uniform1f(location, *value);
        }
    }
}

impl UniformValue for i32 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::Uniform1i(location, *value);
        }
    }
}

impl UniformValue for glm::Vec2 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::Uniform2f(location, value.x, value.y);
        }
    }
}

impl UniformValue for glm::Vec3 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::Uniform3f(location, value.x, value.y, value.z);
        }
    }
}

impl UniformValue for glm::Vec4 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::Uniform4f(location, value.x, value.y, value.z, value.w);
        }
    }
}

impl UniformValue for glm::Mat4 {
    fn set_uniform(location: gl::types::GLint, value: &Self) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, 0, value.as_ptr());
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    Program,
}

/// A program which is a combination of a vertex, fragment and possibly a geometry shader.
#[derive(Debug, Clone, Copy)]
pub struct Shader {
    id: u32,
}

impl Shader {
    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn compile_shader(vertex_src: &str, frag_src: &str, geo_src: Option<&str>) -> Result<Self> {
        let vertex_src: Vec<i8> = vertex_src.bytes().map(|x| x as i8).collect();
        let frag_src: Vec<i8> = frag_src.bytes().map(|x| x as i8).collect();
        let geo_src: Option<Vec<i8>> = geo_src.map(|x| x.bytes().map(|x| x as i8).collect());
        unsafe {
            let s_vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(s_vertex, 1, &(vertex_src.as_ptr()), std::ptr::null());
            gl::CompileShader(s_vertex);
            let mut success = check_compile_errors(s_vertex, ShaderType::Vertex);

            let s_frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(s_frag, 1, &(frag_src.as_ptr()), std::ptr::null());
            gl::CompileShader(s_frag);
            success = success && check_compile_errors(s_frag, ShaderType::Fragment);

            let s_geo = if let Some(geo_src) = geo_src {
                let s_geo = gl::CreateShader(gl::GEOMETRY_SHADER);
                gl::ShaderSource(s_geo, 1, &(geo_src.as_ptr()), std::ptr::null());
                gl::CompileShader(s_geo);
                success = success && check_compile_errors(s_geo, ShaderType::Geometry);
                Some(s_geo)
            } else {
                None
            };

            let program = gl::CreateProgram();
            gl::AttachShader(program, s_vertex);
            gl::AttachShader(program, s_frag);
            if let Some(s_geo) = s_geo {
                gl::AttachShader(program, s_geo);
            }
            gl::LinkProgram(program);

            let shader = Shader { id: program };

            success = success && check_compile_errors(shader.id, ShaderType::Program);

            gl::DeleteShader(s_vertex);
            gl::DeleteShader(s_frag);
            if let Some(s_geo) = s_geo {
                gl::DeleteShader(s_geo);
            }

            if !success {
                return Err(anyhow!("Couldn't compile or link shader"));
            }

            Ok(shader)
        }
    }

    pub fn from_paths(
        vertex_path: &Path,
        frag_path: &Path,
        geo_path: Option<&Path>,
    ) -> Result<Self> {
        let vertex_src = fs::read_to_string(vertex_path)?;
        let frag_src = fs::read_to_string(frag_path)?;
        let geo_src = if let Some(geo_path) = geo_path {
            Some(fs::read_to_string(geo_path)?)
        } else {
            None
        };

        Self::compile_shader(&vertex_src, &frag_src, geo_src.as_deref())
    }

    fn find(&self, name: &str) -> gl::types::GLint {
        let c_string = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, c_string.as_ptr()) }
    }

    pub fn set_uniform<T: UniformValue>(&self, name: &str, value: &T) {
        let loc = self.find(name);
        T::set_uniform(loc, value);
    }
}

fn check_compile_errors(id: u32, shader_type: ShaderType) -> bool {
    let mut success: i32 = 0;
    let mut info_log: Vec<i8> = vec![0; 1024];
    let mut length: i32 = 0;
    unsafe {
        match shader_type {
            ShaderType::Vertex | ShaderType::Fragment | ShaderType::Geometry => {
                gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
                gl::GetShaderInfoLog(id, 1024, &mut length, info_log.as_mut_ptr());
            }
            ShaderType::Program => {
                gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
                gl::GetProgramInfoLog(id, 1024, &mut length, info_log.as_mut_ptr());
            }
        }
        let info = String::from_utf8(info_log.into_iter().map(|b| b as u8).collect())
            .unwrap_or(String::from("Unkown error"));
        if success == 0 {
            error!(
                "ERROR::SHADER: Type {:?} Error flag: {} {}",
                shader_type, success, info
            );
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use glfw::Context;

    use super::*;

    fn init_window() -> glfw::PWindow {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

        let (mut window, _events) = glfw
            .create_window(100, 100, "App", glfw::WindowMode::Windowed)
            .unwrap();

        window.make_current();
        window.set_key_polling(true);

        gl::load_with(|ptr| {
            let f = window.get_proc_address(ptr);
            match f {
                Some(f) => f as *const _,
                None => std::ptr::null(),
            }
        });

        window
    }

    #[test]
    fn can_load_rectangle_rendering_shader() {
        let vertex_src = include_str!("../shaders/rounded_rect.vs");
        let frag_src = include_str!("../shaders/rounded_rect.frag");

        let _window = init_window();

        let shader = Shader::compile_shader(vertex_src, frag_src, None);
        assert!(shader.is_ok(), "Shader should compile successfully");
    }
}
