use crate::Pipe;
use std::marker::PhantomData;

pub struct Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    pipe0: P0,
    pipe1: P1,
}

impl<P0, P1> Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    pub fn new(pipe0: P0, pipe1: P1) -> Self {
        Connector { pipe0, pipe1 }
    }
}

impl<P0, P1> Pipe for Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    type InputItem = P0::InputItem;
    type OutputItem = P1::OutputItem;

    #[inline]
    fn next(&mut self, input: Self::InputItem) -> Self::OutputItem {
        self.pipe1.next(self.pipe0.next(input))
    }
}

pub struct Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    pipe: P,
}

impl<P> Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    pub fn new(pipe: P) -> Self {
        Self { pipe }
    }
}

impl<P> Pipe for Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    type InputItem = P::InputItem;
    type OutputItem = (P::InputItem, P::OutputItem);

    #[inline]
    fn next(&mut self, input: P::InputItem) -> (P::InputItem, P::OutputItem) {
        (input.clone(), self.pipe.next(input))
    }
}

pub struct Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    function: F,
    input: PhantomData<I>,
    output: PhantomData<O>,
}

impl<I, O, F> Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    pub fn new(function: F) -> Self {
        Self {
            function,
            input: PhantomData,
            output: PhantomData,
        }
    }
}

impl<I, O, F> Pipe for Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    type InputItem = I;
    type OutputItem = O;

    #[inline]
    fn next(&mut self, input: I) -> O {
        (self.function)(input)
    }
}

pub struct OptionMap<P>
where
    P: Pipe,
{
    pipe: P,
}

impl<P: Pipe> OptionMap<P> {
    pub fn new(pipe: P) -> Self {
        Self { pipe }
    }
}

impl<P> Pipe for OptionMap<P>
where
    P: Pipe,
{
    type InputItem = Option<P::InputItem>;
    type OutputItem = Option<P::OutputItem>;

    #[inline]
    fn next(&mut self, item: Option<P::InputItem>) -> Option<P::OutputItem> {
        item.map(|item| self.pipe.next(item))
    }
}

pub struct Enumerate<P>
where
    P: Pipe,
{
    pipe: P,
    progress: usize,
}

impl<P: Pipe> Enumerate<P> {
    pub fn new(pipe: P) -> Self {
        Enumerate { pipe, progress: 0 }
    }
}

impl<P: Pipe> Pipe for Enumerate<P> {
    type InputItem = P::InputItem;
    type OutputItem = (usize, P::OutputItem);

    #[inline]
    fn next(&mut self, item: P::InputItem) -> (usize, P::OutputItem) {
        let next_item = self.pipe.next(item);
        let index = self.progress;
        self.progress += 1;
        (index, next_item)
    }
}
