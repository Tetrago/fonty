use fonty::Glyph;
use fonty::prelude::*;
use fonty_gfx::*;
use nalgebra::Matrix4;
use std::cell::RefCell;
use std::env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[2].len() != 1 {
        return Err("Invalid arguments".into());
    }

    let ttf = RefCell::new(fonty::open(&std::path::Path::new(&args[1]))?);
    let character = args[2].chars().next().unwrap();

    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 0);

    let window = video
        .window("Glyph", 1024, 1024)
        .opengl()
        .resizable()
        .build()?;
    let _context = window.gl_create_context()?;

    gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let get_ortho = |width: i32, height: i32| {
        let width = width as f32;
        let height = height as f32;
        let factor = width.min(height);

        Matrix4::new_orthographic(
            0.0,
            width / factor,
            -height / factor / 4.0,
            height / factor / 4.0 * 3.0,
            -1.0,
            1.0,
        )
    };

    let mut ortho = {
        let (width, height) = window.size();
        get_ortho(width as i32, height as i32)
    };

    let glyph = ttf.borrow_mut().glyph(character)?;
    let mut factory = OutlineFactory::new(&ttf);
    let shape = factory.get(character)?;

    unsafe {
        let mut event_pump = sdl.event_pump()?;
        'main: loop {
            for event in event_pump.poll_iter() {
                use sdl2::event::Event;
                use sdl2::event::WindowEvent;

                match event {
                    Event::Quit { .. } => break 'main,
                    Event::Window { win_event, .. } => match win_event {
                        WindowEvent::Resized(width, height)
                        | WindowEvent::SizeChanged(width, height) => {
                            ortho = get_ortho(width, height);
                            gl::Viewport(0, 0, width, height);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            shape.borrow().draw(0.0, 0.0, 0.0005, &ortho);

            window.gl_swap_window();
        }
    }

    Ok(())
}
