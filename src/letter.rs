use crate::Result;
use crate::Shader;
use crate::compile_shader;
use fonty::Glyph;
use nalgebra::Matrix4;
use nalgebra::Translation3;
use std::cell::RefCell;
use std::mem::size_of;
use std::ptr;
use std::rc::Rc;

pub struct Builder {
    shader: Rc<RefCell<Shader>>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            shader: Rc::new(RefCell::new(
                compile_shader!(VERTEX_SHADER => "letter_vert.glsl", FRAGMENT_SHADER => "letter_frag.glsl")
                .unwrap(),
            )),
        }
    }

    pub fn of(&self, glyph: Glyph) -> Result<Letter> {
        Letter::new(self.shader.clone(), glyph)
    }
}

pub struct Letter {
    shader: Rc<RefCell<Shader>>,
    vao: u32,
    vbo: u32,
    ibo: u32,
    count: i32,
}

impl Letter {
    pub(self) fn new(shader: Rc<RefCell<Shader>>, glyph: Glyph) -> Result<Self> {
        if let Glyph::Simple {
            end_points, points, ..
        } = glyph
        {
            unsafe {
                let mut vao = 0u32;
                let mut vbo = 0u32;
                let mut ibo = 0u32;

                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);
                gl::GenBuffers(1, &mut ibo);

                gl::BindVertexArray(vao);

                let points: Vec<_> = points
                    .iter()
                    .map(|(x, y, _)| (*x as f32, *y as f32))
                    .collect();

                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (points.len() * size_of::<f32>() * 2) as isize,
                    points.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);

                let mut indices = Vec::<u32>::new();
                let mut start = 0u32;

                for end in end_points {
                    indices.push(start);

                    for i in start..=end as u32 {
                        indices.push(i);
                        indices.push(i);
                    }

                    indices.push(start);
                    start = end as u32 + 1;
                }

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (indices.len() * size_of::<u32>()) as isize,
                    indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

                Ok(Self {
                    shader,
                    vao,
                    vbo,
                    ibo,
                    count: indices.len() as i32,
                })
            }
        } else {
            Err("Unsupported glyph".into())
        }
    }

    pub fn draw(&self, x: f32, y: f32, scale: f32, matrix: &Matrix4<f32>) {
        let matrix =
            matrix * Translation3::new(x, y, 0.0).to_homogeneous() * Matrix4::new_scaling(scale);

        unsafe {
            let mut shader = self.shader.borrow_mut();

            gl::UseProgram(shader.handle());

            gl::UniformMatrix4fv(
                shader.get_uniform_location("u_Mvp"),
                1,
                gl::FALSE,
                matrix.as_slice().as_ptr(),
            );

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::LINES, self.count, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Letter {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.ibo);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}
