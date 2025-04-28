use crate::Error;
use crate::Result;
use crate::factory;
use crate::outline::Instance;
use fonty::Glyph;
use fonty::OutlineFlag;
use fonty::Ttf;
use nalgebra::Matrix4;
use nalgebra::Translation3;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

pub struct Shape {
    instance: Rc<RefCell<Instance>>,
    vao: u32,
    vbo: u32,
    ibo: u32,
    count: i32,
}

impl Shape {
    pub fn draw(&self, x: f32, y: f32, scale: f32, matrix: &Matrix4<f32>) {
        let matrix =
            matrix * Translation3::new(x, y, 0.0).to_homogeneous() * Matrix4::new_scaling(scale);

        unsafe {
            let shader = &mut self.instance.borrow_mut().shader;

            gl::UseProgram(shader.handle());

            gl::UniformMatrix4fv(
                shader.get_uniform_location("u_Mvp"),
                1,
                gl::FALSE,
                matrix.as_slice().as_ptr(),
            );

            gl::BindVertexArray(self.vao);
            gl::PatchParameteri(gl::PATCH_VERTICES, 3);
            gl::DrawElements(gl::PATCHES, self.count, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &mut self.ibo);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteVertexArrays(1, &mut self.vao);
        }
    }
}

impl factory::Shape<Instance> for Shape {
    fn new(ttf: &mut dyn Ttf, instance: Rc<RefCell<Instance>>, c: char) -> Result<Self> {
        if let Glyph::Simple {
            end_points, points, ..
        } = ttf.glyph(c)?
        {
            unsafe {
                let mut vao = 0u32;
                let mut vbo = 0u32;
                let mut ibo = 0u32;

                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);
                gl::GenBuffers(1, &mut ibo);

                gl::BindVertexArray(vao);

                let mut vertices = Vec::<(f32, f32, i32)>::new();
                let mut next_end = 0;

                let mut loop_ends = Vec::<(usize, usize)>::with_capacity(end_points.len());
                let mut last_end = 0;

                let offset = if points.is_empty() || OutlineFlag::OnCurve.test(points[0].2) {
                    0
                } else {
                    1
                };

                for i in 0..points.len() {
                    let (x, y, flags) = points[i];
                    let on_curve = OutlineFlag::OnCurve.test(flags);

                    vertices.push((x as f32, y as f32, if on_curve { 1 } else { 0 }));

                    let idx = if i as u16 == end_points[next_end] {
                        loop_ends.push((vertices.len() - 1, last_end));
                        last_end = vertices.len();

                        let idx = if next_end == 0 {
                            0
                        } else {
                            end_points[next_end - 1] as usize + 1
                        };

                        next_end += 1;
                        idx
                    } else {
                        i + 1
                    };

                    let (bx, by, f) = points[idx];

                    if on_curve == OutlineFlag::OnCurve.test(f) {
                        let midpoint = (
                            (x + bx) as f32 * 0.5,
                            (y + by) as f32 * 0.5,
                            if on_curve { 1 } else { 0 },
                        );
                        vertices.push(midpoint);

                        if last_end == vertices.len() - 1 {
                            last_end += 1;
                        }
                    }

                    if next_end == end_points.len() {
                        break;
                    }
                }

                let stride = size_of::<f32>() * 2 + size_of::<i32>();

                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * stride) as isize,
                    vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                let mut indices = Vec::<u32>::new();
                let mut i = offset;
                let mut loop_idx = 0;

                while i < vertices.len() {
                    if i == loop_ends[loop_idx].0 {
                        if offset == 1 {
                            indices.push(i as u32);
                            indices.push(loop_ends[loop_idx].1 as u32);
                            indices.push(loop_ends[loop_idx].1 as u32 + 1);
                        } else {
                            indices.push(i as u32);
                            indices.push(i as u32 + 1);
                            indices.push(loop_ends[loop_idx].1 as u32);
                        }

                        loop_idx += 1;
                    } else {
                        indices.push(i as u32);
                        indices.push(i as u32 + 1);
                        indices.push((i as u32 + 2) % vertices.len() as u32);
                    }

                    i += 2;
                }

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (indices.len() * size_of::<u32>()) as isize,
                    indices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride as i32, ptr::null());
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(
                    1,
                    1,
                    gl::INT,
                    gl::FALSE,
                    stride as i32,
                    (size_of::<f32>() * 2) as *const _,
                );

                gl::BindBuffer(gl::ARRAY_BUFFER, 0);

                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

                Ok(Self {
                    instance,
                    vao,
                    vbo,
                    ibo,
                    count: indices.len() as i32,
                })
            }
        } else {
            Err(Error::UnsupportedFeature)
        }
    }
}
