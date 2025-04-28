use crate::Error;
use crate::Result;
use std::collections::HashMap;

struct Part {
    pub handle: u32,
}

impl Part {
    pub fn new(ty: u32, source: &[u8]) -> Result<Self> {
        unsafe {
            let handle = gl::CreateShader(ty);

            let src = source.as_ptr() as *const i8;
            let len = source.len() as i32;

            gl::ShaderSource(handle, 1, &src as *const *const i8, &len as *const _);
            gl::CompileShader(handle);

            let mut res = 0i32;
            gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut res);

            if res != gl::TRUE as i32 {
                let mut log = vec![0u8; 256];
                let mut length = 0i32;

                gl::GetShaderInfoLog(handle, 256, &mut length, log.as_ptr() as *mut _);
                log.truncate(length as usize);

                gl::DeleteShader(handle);
                Err(Error::ShaderCompilationFailed(
                    String::from_utf8(log).unwrap(),
                ))
            } else {
                Ok(Part { handle })
            }
        }
    }
}

impl Drop for Part {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.handle);
        }
    }
}

pub struct Shader {
    handle: u32,
    uniforms: HashMap<String, i32>,
}

impl Shader {
    pub fn new(parts: &[(u32, &[u8])]) -> Result<Self> {
        unsafe {
            let handle = gl::CreateProgram();
            let mut store = Vec::<Part>::with_capacity(parts.len());

            for (ty, source) in parts {
                let part = Part::new(*ty, *source)?;
                gl::AttachShader(handle, part.handle);
                store.push(part);
            }

            gl::LinkProgram(handle);

            for part in store {
                gl::DetachShader(handle, part.handle);
            }

            let mut res = 0i32;
            gl::GetProgramiv(handle, gl::LINK_STATUS, &mut res);

            if res != gl::TRUE as i32 {
                let mut log = vec![0u8; 256];
                let mut length = 0i32;

                gl::GetProgramInfoLog(handle, 256, &mut length, log.as_ptr() as *mut _);
                log.truncate(length as usize);

                gl::DeleteProgram(handle);
                Err(Error::ShaderCompilationFailed(
                    String::from_utf8(log).unwrap(),
                ))
            } else {
                Ok(Shader {
                    handle,
                    uniforms: HashMap::new(),
                })
            }
        }
    }

    pub fn get_uniform_location(&mut self, name: &str) -> i32 {
        if let Some(location) = self.uniforms.get(name) {
            *location
        } else {
            unsafe {
                let str = std::ffi::CString::new(name).unwrap();
                let location = gl::GetUniformLocation(self.handle, str.as_ptr());

                self.uniforms.insert(name.to_owned(), location);
                location
            }
        }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}

macro_rules! compile_shader {
    ({$($ty:ident => $source:literal),+ $(,)?}) => {
        {
            let parts = [$((gl::$ty, include_bytes!($source) as &[u8])),+];
            Shader::new(&parts)
        }
    }
}

pub(crate) use compile_shader;
