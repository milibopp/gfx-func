//! Drawable elements

use gfx::Resources;
use gfx::batch::Batch;

use command::{ Commands, Command };


/// Abstraction of a drawable element.
pub trait Element<R: Resources> {
    /// An iterator over draw commands
    fn commands(&self) -> Commands<R>;
}

impl<B: Batch<R>, R: Resources> Element<R> for B {
    fn commands(&self) -> Commands<R> {
        Commands::Single(Command::Draw(self))
    }
}
