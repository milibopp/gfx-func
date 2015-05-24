//! Draw commands

use gfx::{ Resources, ClearData };
use gfx::batch::Batch;


/// A simple draw command
pub enum Command<'a, R: Resources> {
    /// Clear the screen as specified
    Clear(ClearData),

    /// Draw a batch represented as a trait-object reference
    Draw(&'a Batch<R>),
}


/// Iterator over draw commands
pub enum Commands<'a, R: Resources> {
    /// Empty iterator
    Empty,

    /// An iterator over a single command
    Single(Command<'a, R>),

    /// An arbitrary boxed iterator over commands
    Iter(Box<Iterator<Item=Command<'a, R>> + 'a>),
}

impl<'a, R: Resources> Iterator for Commands<'a, R> {
    type Item = Command<'a, R>;

    fn next(&mut self) -> Option<Command<'a, R>> {
        use std::mem;
        use self::Commands::*;
        let mut tmp = Empty;
        mem::swap(self, &mut tmp);
        let (ret, new) = match tmp {
            Empty => (None, Empty),
            Single(cmd) => (Some(cmd), Empty),
            Iter(mut iter) => (iter.next(), Iter(iter)),
        };
        *self = new;
        ret
    }
}
