use crate::Pipe;
use std::marker::PhantomData;
use std::ops::Shr;

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

    fn next(&mut self, item: I) -> P::OutputItem {
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

    fn next(&mut self, item: P::InputItem) -> O {
        (self.mapper)(self.pipe.next(item))
    }
}

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

    fn next(&mut self, item: Option<P::InputItem>) -> Option<P::OutputItem> {
        item.map(|item| self.pipe.next(item))
    }
}

pub struct PipeConstraint<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    pipe: P,
    item: PhantomData<(I, O)>,
}

impl<I, O, P> PipeConstraint<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    pub fn new(pipe: P) -> Self {
        Self {
            pipe,
            item: PhantomData,
        }
    }
}

impl<I, O, P> Pipe for PipeConstraint<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    type InputItem = I;
    type OutputItem = O;

    fn next(&mut self, item: I) -> O {
        self.pipe.next(item)
    }
}

impl<I, M, P0, P1> Shr<P1> for PipeConstraint<I, M, P0>
where
    P0: Pipe<InputItem=I, OutputItem=M>,
    P1: Pipe<InputItem=M>
{
    type Output = PipeConstraint<P0::InputItem, P1::OutputItem, Connector<P0, P1>>;

    fn shr(self, other: P1) -> Self::Output {
        Self::Output::new(Connector::new(self.pipe, other))
    }
}
