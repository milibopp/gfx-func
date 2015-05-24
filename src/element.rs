//! Drawable elements

use gfx::{ Resources, ClearData, batch };

use command::{ Commands, Command };


/// Abstraction of a drawable element.
pub trait Element<R: Resources> {
    /// An iterator over draw commands
    fn commands(&self) -> Commands<R>;
}


/// A single batch.
#[derive(Clone)]
pub struct Batch<B>(pub B);

impl<B: batch::Batch<R>, R: Resources> Element<R> for Batch<B> {
    fn commands(&self) -> Commands<R> {
        Commands::Single(Command::Draw(&self.0))
    }
}


/// An element preceded by clearing the screen
#[derive(Clone)]
pub struct Cleared<E> {
    clear: ClearData,
    element: E,
}

impl<E> Cleared<E> {
    pub fn new(clear: ClearData, element: E) -> Cleared<E> {
        Cleared { clear: clear, element: element }
    }
}

impl<E: Element<R>, R: Resources> Element<R> for Cleared<E>
    where R::Buffer: 'static, R::ArrayBuffer: 'static, R::Shader: 'static,
          R::Program: 'static, R::FrameBuffer: 'static, R::Surface: 'static,
          R::Sampler: 'static, R::Texture: 'static, R: 'static,
{
    fn commands(&self) -> Commands<R> {
        Commands::Iter(Box::new(
            Some(Command::Clear(self.clear))
                .into_iter()
                .chain(self.element.commands())
        ))
    }
}
