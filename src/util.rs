use crate::Pipe;
use std::marker::PhantomData;

pub struct PreMap<I, P, F>
where
    P: Pipe,
    F: Fn(I) -> P::InputItem,
{
    input: PhantomData<I>,
    mapper: F,
    pipe: P,
}

impl<I, P, F> PreMap<I, P, F>
where
    P: Pipe,
    F: Fn(I) -> P::InputItem,
{
    pub fn new(pipe: P, mapper: F) -> Self {
        Self {
            input: PhantomData,
            mapper,
            pipe,
        }
    }
}

impl<I, P, F> Pipe for PreMap<I, P, F>
where
    P: Pipe,
    F: Fn(I) -> P::InputItem,
{
    type InputItem = I;
    type OutputItem = P::OutputItem;

    fn next(&mut self, item: I) -> Option<P::OutputItem> {
        self.pipe.next((self.mapper)(item))
    }
}

pub struct PostMap<O, P, F>
where
    P: Pipe,
    F: Fn(P::OutputItem) -> O,
{
    output: PhantomData<O>,
    mapper: F,
    pipe: P,
}

impl<O, P, F> PostMap<O, P, F>
where
    P: Pipe,
    F: Fn(P::OutputItem) -> O,
{
    pub fn new(pipe: P, mapper: F) -> Self {
        Self {
            output: PhantomData,
            mapper,
            pipe,
        }
    }
}

impl<O, P, F> Pipe for PostMap<O, P, F>
where
    P: Pipe,
    F: Fn(P::OutputItem) -> O,
{
    type InputItem = P::InputItem;
    type OutputItem = O;

    fn next(&mut self, item: P::InputItem) -> Option<O> {
        Some((self.mapper)(self.pipe.next(item)?))
    }
}

pub struct PipeConnector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    pipe0: P0,
    pipe1: P1,
}

impl<P0, P1> PipeConnector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    pub fn new(pipe0: P0, pipe1: P1) -> Self {
        Self { pipe0, pipe1 }
    }
}

impl<P0, P1> Pipe for PipeConnector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    type InputItem = P0::InputItem;
    type OutputItem = P1::OutputItem;

    fn next(&mut self, input: Self::InputItem) -> Option<Self::OutputItem> {
        let intermediate = self.pipe0.next(input)?;
        self.pipe1.next(intermediate)
    }
}
