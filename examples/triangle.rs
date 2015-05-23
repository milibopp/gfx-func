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

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use carboxyl::Signal;
use carboxyl_window::{ SourceWindow, EventSource };
use window::WindowSettings;
use shader_version::OpenGL;
use glutin_window::GlutinWindow;
use gfx::traits::FactoryExt;
use gfx::{ Stream, Resources, ClearData };
use gfx::batch::{ Batch, OwnedBatch };
use gfx::shade::{ ShaderParam, ParameterError };
use gfx::device::shade::ProgramInfo;
use gfx::render::ParamStorage;
use gfx_func::Element;

pub mod shared_win;


gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
    a_Color@ color: [f32; 3],
});

struct ParamWrapper<T> {
    params: Arc<T>,
}

impl<T> Clone for ParamWrapper<T> {
    fn clone(&self) -> ParamWrapper<T> {
        ParamWrapper { params: self.params.clone() }
    }
}

impl<T: ShaderParam> ShaderParam for ParamWrapper<T> {
    type Resources = T::Resources;
    type Link = T::Link;
    fn create_link(maybe_self: Option<&ParamWrapper<T>>, info: &ProgramInfo) -> Result<T::Link, ParameterError> {
        ShaderParam::create_link(maybe_self.map(|x| &*x.params), info)
    }
    fn fill_params(&self, link: &T::Link, storage: &mut ParamStorage<T::Resources>) {
        <T as ShaderParam>::fill_params(&self.params, link, storage)
    }
}

gfx_parameters!(Params {});


fn run_from_source<R, W, E, F, S>(source: &mut SourceWindow<W>, stream: &mut S,
                                  mut render: F, element: Signal<E>)
    where R: Resources,
          W: EventSource,
          E: Element<R> + Clone + Send + Sync + 'static,
          S: Stream<R>,
          F: FnMut(&mut S),
{
    use gfx::extra::stream::Stream;
    source.run(|| {
        stream.clear(gfx::ClearData {
            color: [0.3, 0.3, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        });
        let current = element.sample();
        for batch in current.batches() {
            let _: &Batch<R> = batch;
            stream.draw(batch).unwrap();
        }
        render(stream)
    })
}


fn main() {
    const GLVERSION: OpenGL = OpenGL::_2_1;
    let settings = WindowSettings::new("gfx + carboxyl_window", (640, 480));
    let window = Rc::new(RefCell::new(GlutinWindow::new(GLVERSION, settings)));
    let (mut stream, mut device, mut factory) = shared_win::init_shared(window.clone());
    let mut source = SourceWindow::new(window.clone(), 10_000_000);

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
        let data = ParamWrapper { params: Arc::new(Params { _r: std::marker::PhantomData }) };
        data.clone();
        OwnedBatch::new(mesh, program, data).unwrap()
    };

    run_from_source(
        &mut source, &mut stream,
        |s| s.present(&mut device),
        Signal::new(batch)
    );
}
