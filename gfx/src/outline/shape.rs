use crate::Error;
use crate::Result;
use crate::factory;
use crate::outline::Instance;
use fonty::ComponentFlag;
use fonty::Glyph;
use fonty::OutlineFlag;
use fonty::OutlineFlags;
use fonty::Ttf;
use nalgebra::Matrix4;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;

pub struct Shape {
    instance: Rc<RefCell<Instance>>,
    vao: u32,
    vbo: u32,
    ibo: u32,
    count: i32,
    bounding_box: (f32, f32, f32, f32),
}

impl Shape {
    pub fn draw(
        &self,
        stroke_color: (f32, f32, f32, f32),
        stroke_size: f32,
        matrix: &Matrix4<f32>,
    ) {
        unsafe {
            let shader = &mut self.instance.borrow_mut().shader;

            gl::UseProgram(shader.handle());

            gl::UniformMatrix4fv(
                shader.get_uniform_location("u_Mvp"),
                1,
                gl::FALSE,
                matrix.as_slice().as_ptr(),
            );

            gl::Uniform4fv(
                shader.get_uniform_location("u_Color"),
                1,
                &stroke_color as *const _ as *const _,
            );

            gl::Uniform1f(shader.get_uniform_location("u_Size"), stroke_size);

            gl::PatchParameteri(gl::PATCH_VERTICES, 3);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::PATCHES, self.count, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }

    pub fn bounding_box(&self) -> (f32, f32, f32, f32) {
        self.bounding_box
    }

    pub fn size(&self) -> (f32, f32) {
        (
            self.bounding_box.2 - self.bounding_box.0,
            self.bounding_box.3 - self.bounding_box.1,
        )
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

fn build_mesh(points: &[(i16, i16, OutlineFlags)]) -> (Vec<(f32, f32, i32)>, Vec<u32>) {
    let mut vertices = Vec::<(f32, f32, i32)>::new();
    let mut indices = Vec::<u32>::new();

    for i in 0..points.len() {
        let (x, y, flags) = points[i];
        let on_curve = OutlineFlag::OnCurve.test(flags);

        vertices.push((x as f32, y as f32, on_curve as i32));

        let (bx, by, flags) = points[(i + 1) % points.len()];

        if on_curve == OutlineFlag::OnCurve.test(flags) {
            let midpoint = (
                (x as f32 + bx as f32) * 0.5,
                (y as f32 + by as f32) * 0.5,
                on_curve as i32,
            );

            vertices.push(midpoint);
        }
    }

    if !OutlineFlag::OnCurve.test(points[0].2) {
        vertices.rotate_right(1);
    }

    for i in (0..vertices.len()).step_by(2) {
        indices.push(i as u32);
        indices.push(i as u32 + 1);
        indices.push((i as u32 + 2) % vertices.len() as u32);
    }

    (vertices, indices)
}

fn build_glyph(ttf: &mut dyn Ttf, glyph: &Glyph) -> Result<(Vec<(f32, f32, i32)>, Vec<u32>)> {
    match glyph {
        Glyph::Simple {
            end_points, points, ..
        } => {
            let mut vertices = Vec::<(f32, f32, i32)>::new();
            let mut indices = Vec::<u32>::new();

            let mut start = 0usize;

            for &end in end_points {
                let (v, i) = build_mesh(&points[start..=end as usize]);
                start = end as usize + 1;

                indices.extend(i.iter().map(|i| i + vertices.len() as u32));
                vertices.extend(
                    v.into_iter()
                        .map(|(x, y, c)| (x * ttf.scale() as f32, y * ttf.scale() as f32, c)),
                );
            }

            Ok((vertices, indices))
        }
        Glyph::Compound { components, .. } => {
            let mut vertices = Vec::<(f32, f32, i32)>::new();
            let mut indices = Vec::<u32>::new();
            let mut points = Vec::<(f32, f32)>::new();

            for component in components {
                let glyph = ttf.glyph_at(component.index)?;
                let (v, i) = build_glyph(ttf, &glyph)?;

                let (dx, dy) = if ComponentFlag::ArgsAreXyValues.test(component.flags) {
                    Ok((component.offset.0 as f32, component.offset.1 as f32))
                } else if let Glyph::Simple { points: p, .. } = &glyph {
                    let global_point = points[component.offset.0 as usize];
                    let (x, y, _) = p[component.offset.1 as usize];

                    Ok(component.subtract((x as f32, y as f32), global_point))
                } else {
                    Err(Error::UnsupportedNestedComponent)
                }?;

                if let Glyph::Simple { points: p, .. } = &glyph {
                    points.extend(
                        p.iter()
                            .map(|&(x, y, _)| component.transform(x as f32, y as f32, dx, dy)),
                    );
                } else {
                    return Err(Error::UnsupportedNestedComponent);
                }

                let i: Vec<u32> = i.into_iter().map(|i| i + vertices.len() as u32).collect();
                let v: Vec<(f32, f32, i32)> = v
                    .into_iter()
                    .map(|(x, y, c)| {
                        let (x, y) = component.transform(x, y, dx, dy);
                        (x, y, c)
                    })
                    .collect();

                indices.extend(i);
                vertices.extend(v);
            }

            Ok((vertices, indices))
        }
    }
}

impl factory::Shape<Instance> for Shape {
    fn new(ttf: &mut dyn Ttf, instance: Rc<RefCell<Instance>>, c: char) -> Result<Self> {
        let glyph = ttf.glyph(c)?;
        let (vertices, indices) = build_glyph(ttf, &glyph)?;
        let stride = size_of::<f32>() * 2 + size_of::<i32>();

        let bounding_box = {
            let mut min = (0.0, 0.0);
            let mut max = (0.0, 0.0);

            for vertex in &vertices {
                min.0 = f32::min(min.0, vertex.0);
                min.1 = f32::min(min.1, vertex.1);

                max.0 = f32::max(max.0, vertex.0);
                max.1 = f32::max(max.1, vertex.1);
            }

            (min.0, min.1, max.0, max.1)
        };

        unsafe {
            let mut vao = 0u32;
            let mut vbo = 0u32;
            let mut ibo = 0u32;

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ibo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * stride) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

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
                bounding_box,
            })
        }
    }
}
