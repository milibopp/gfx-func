extern crate gfx;

use gfx::{ Resources, ParamStorage };
use gfx::device::handle::Program;
use gfx::batch::{ Batch, BatchData };


pub trait DynamicBatch<R: Resources> {
    fn get_data(&self) -> Result<BatchData<R>, String>;
    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, String>;
}

impl<B: Batch<R>, R: Resources> DynamicBatch<R> for B {
    fn get_data(&self) -> Result<BatchData<R>, String> {
        Batch::get_data(self)
            .map_err(|err| format!("{:?}", err))
    }

    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, String> {
        Batch::fill_params(self, values)
            .map_err(|err| format!("{:?}", err))
    }
}

impl<'a, R: Resources> Batch<R> for Box<DynamicBatch<R> + 'a> {
    type Error = String;
    fn get_data(&self) -> Result<BatchData<R>, String> {
        DynamicBatch::get_data(&**self)
    }

    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, String> {
        DynamicBatch::fill_params(&**self, values)
    }
}

impl<'a, R: Resources> Batch<R> for &'a DynamicBatch<R> {
    type Error = String;
    fn get_data(&self) -> Result<BatchData<R>, String> {
        DynamicBatch::get_data(&**self)
    }

    fn fill_params(&self, values: &mut ParamStorage<R>) -> Result<&Program<R>, String> {
        DynamicBatch::fill_params(&**self, values)
    }
}


pub enum Batches<'a, R> {
    Empty,
    Single(&'a DynamicBatch<R>),
    Iter(Box<Iterator<Item=&'a DynamicBatch<R>> + 'a>),
}

impl<'a, R> Iterator for Batches<'a, R> {
    type Item = &'a DynamicBatch<R>;

    fn next(&mut self) -> Option<&'a DynamicBatch<R>> {
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


pub trait Element<R: Resources> {
    fn batches(&self) -> Batches<R>;
}

impl<B: Batch<R>, R: Resources> Element<R> for B {
    fn batches(&self) -> Batches<R> {
        Batches::Single(self)
    }
}
