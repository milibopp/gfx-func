//! Drawable elements

use gfx::Resources;
use gfx::batch::Batch;


/// Abstraction of a drawable element.
pub trait Element<R: Resources> {
    /// Yield an 
    fn batches(&self) -> Batches<R>;
}

impl<B: Batch<R>, R: Resources> Element<R> for B {
    fn batches(&self) -> Batches<R> {
        Batches::Single(self)
    }
}


pub enum Batches<'a, R> {
    Empty,
    Single(&'a Batch<R>),
    Iter(Box<Iterator<Item=&'a Batch<R>> + 'a>),
}

impl<'a, R> Iterator for Batches<'a, R> {
    type Item = &'a Batch<R>;

    fn next(&mut self) -> Option<&'a Batch<R>> {
        use std::mem;
        use self::Batches::*;
        let mut tmp = Empty;
        mem::swap(self, &mut tmp);
        let (ret, new) = match tmp {
            Empty => (None, Empty),
            Single(batch) => (Some(batch), Empty),
            Iter(mut iter) => (iter.next(), Iter(iter)),
        };
        *self = new;
        ret
    }
}
