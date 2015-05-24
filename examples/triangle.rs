#[macro_use(gfx_vertex, gfx_parameters)]
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate carboxyl;
extern crate carboxyl_window;
extern crate window;
extern crate input;
extern crate shader_version;
extern crate glutin_window;
extern crate gfx_func;

use std::sync::{ Arc, RwLock };
use carboxyl::Signal;
use carboxyl_window::{ SourceWindow, RunnableWindow };
use window::WindowSettings;
use shader_version::OpenGL;
use glutin_window::GlutinWindow;
use gfx::traits::FactoryExt;
use gfx::{ Stream, Resources, ClearData };
use gfx::batch::OwnedBatch;
use gfx_func::Element;
use gfx_func::element::{ Batch, Cleared };

pub mod shared_win;


gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
    a_Color@ color: [f32; 3],
});


pub fn draw_element<R, E, F, S>(stream: &mut S, mut render: F, element: &Signal<E>)
    where R: Resources, S: Stream<R>, F: FnMut(&mut S),
          E: Element<R> + Clone + Send + Sync + 'static,
{
    let current = element.sample();
    for cmd in current.commands() {
        use gfx_func::command::Command::*;
        match cmd {
            Clear(data) => stream.clear(data),
            Draw(batch) => stream.draw(batch).unwrap(),
        }
    }
    render(stream);
}


fn main() {
    const GLVERSION: OpenGL = OpenGL::_2_1;
    let settings = WindowSettings::new("gfx + carboxyl_window", (640, 480));
    let window = Arc::new(RwLock::new(GlutinWindow::new(GLVERSION, settings)));
    let (mut stream, mut device, mut factory) = shared_win::init_shared(window.clone());
    let mut source = SourceWindow::new(window.clone());

    let batch = {
        let vertex_data = [
            Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
        ];
        let mesh = factory.create_mesh(&vertex_data);
        let program = {
            let vs = gfx::ShaderSource {
                glsl_120: Some(include_bytes!("triangle_120.glslv")),
                glsl_150: Some(include_bytes!("triangle_150.glslv")),
                .. gfx::ShaderSource::empty()
            };
            let fs = gfx::ShaderSource {
                glsl_120: Some(include_bytes!("triangle_120.glslf")),
                glsl_150: Some(include_bytes!("triangle_150.glslf")),
                .. gfx::ShaderSource::empty()
            };
            factory.link_program_source(vs, fs).unwrap()
        };
        OwnedBatch::new(mesh, program, None).unwrap()
    };
    let signal = Signal::new(Cleared::new(
        ClearData { color: [0.3, 0.3, 0.3, 1.0], depth: 1.0, stencil: 0 },
        Batch(batch)
    ));

    source.run_with(120.0, || {
        draw_element(&mut stream, |s| s.present(&mut device), &signal);
    });
}
