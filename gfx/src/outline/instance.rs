use crate::Shader;
use crate::compile_shader;
use crate::factory;

pub struct Instance {
    pub shader: Shader,
}

impl factory::Instance for Instance {
    type Shape = crate::outline::Shape;

    fn new() -> Self {
        Self {
            shader: compile_shader!({
                VERTEX_SHADER => "vert.glsl",
                FRAGMENT_SHADER => "frag.glsl",
                TESS_CONTROL_SHADER => "ctrl.glsl",
                TESS_EVALUATION_SHADER => "eval.glsl",
            })
            .unwrap(),
        }
    }
}
