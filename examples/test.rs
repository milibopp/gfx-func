#[macro_use(gfx_vertex)]
extern crate gfx;
extern crate gfx_device_gl;
extern crate glutin;
extern crate carboxyl_window;
extern crate window;
extern crate input;
extern crate shader_version;
extern crate glutin_window;

use std::rc::Rc;
use std::cell::RefCell;
use carboxyl_window::{ SourceWindow, EventSource };
use window::{ WindowSettings };
use shader_version::OpenGL;
use glutin_window::GlutinWindow;
use gfx::traits::{ FactoryExt, ToSlice };
use gfx::{ Stream, Resources, ClearData, ParamStorage };
use gfx::device::handle::Program;
use gfx::shade::ShaderParam;
use gfx::batch::{ RefBatch, RefBatchFull, Batch, Context, OutOfBounds, BatchData };

pub mod shared_win;


gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
    a_Color@ color: [f32; 3],
});


pub trait RefBatchPoly<R: Resources> {
    fn get_data(&self) -> Result<BatchData<R>, OutOfBounds>;
    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, OutOfBounds>;
}

impl<'a, T: ShaderParam + 'a> RefBatchPoly<T::Resources> for RefBatchFull<'a, T> {
    fn get_data(&self) -> Result<BatchData<T::Resources>, OutOfBounds> {
        Batch::get_data(self)
    }
    fn fill_params(&self, values: &mut ParamStorage<T::Resources>) -> Result<&Program<T::Resources>, OutOfBounds> {
        Batch::fill_params(self, values)
    }
}

impl<'a, R: Resources> Batch<R> for Box<RefBatchPoly<R> + 'a> {
    type Error = OutOfBounds;
    fn get_data(&self) -> Result<BatchData<R>, OutOfBounds> {
        RefBatchPoly::get_data(&**self)
    }
    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, OutOfBounds> {
        RefBatchPoly::fill_params(&**self, values)
    }
}

impl<'a, R: Resources> Batch<R> for &'a RefBatchPoly<R> {
    type Error = OutOfBounds;
    fn get_data(&self) -> Result<BatchData<R>, OutOfBounds> {
        RefBatchPoly::get_data(&**self)
    }
    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, OutOfBounds> {
        RefBatchPoly::fill_params(&**self, values)
    }
}


pub trait Element<R: Resources> {
    fn batches<'a>(&'a self, context: &'a Context<R>) -> Box<Iterator<Item=Box<RefBatchPoly<R> + 'a>> + 'a>;
}


impl<T: ShaderParam> Element<T::Resources> for RefBatch<T> {
    fn batches<'a>(&'a self, context: &'a Context<T::Resources>) -> Box<Iterator<Item=Box<RefBatchPoly<T::Resources> + 'a>> + 'a> {
        let b: Box<RefBatchPoly<T::Resources> + 'a> = Box::new((self, context));
        Box::new(Some(b).into_iter())
    }
}


fn run_from_source<R, W, E, F, S>(source: &mut SourceWindow<W>, stream: &mut S,
                                  mut render: F, context: &Context<R>,
                                  element: E)
    where R: Resources,
          W: EventSource,
          E: Element<R>,
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
        for batch in element.batches(context) {
            stream.draw(&batch).unwrap();
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

    let mut context = Context::new();

    let batch = {
        let vertex_data = [
            Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
        ];
        let mesh = factory.create_mesh(&vertex_data);
        let slice = mesh.to_slice(gfx::PrimitiveType::TriangleList);
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
        let state = gfx::DrawState::new();
        context.make_batch(&program, None, &mesh, slice, &state).ok().unwrap()
    };


    run_from_source(
        &mut source, &mut stream,
        |s| s.present(&mut device),
        &mut context,
        batch,
    );
}
