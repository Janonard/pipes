pub mod slice;
pub mod util;

pub trait Pipe: Sized {
    type InputItem;
    type OutputItem;

    fn next(&mut self, item: Self::InputItem) -> Self::OutputItem;

    fn bypass(self) -> util::Bypass<Self>
    where
        Self::InputItem: Clone,
    {
        util::Bypass::new(self)
    }

    fn optional(self) -> util::OptionMap<Self> {
        util::OptionMap::new(self)
    }

    fn constraint(self) -> util::PipeConstraint<Self::InputItem, Self::OutputItem, Self> {
        util::PipeConstraint::new(self)
    }
}

impl<P0, P1> Pipe for (P0, P1)
where
    P0: Pipe,
    P1: Pipe,
{
    type InputItem = (P0::InputItem, P1::InputItem);
    type OutputItem = (P0::OutputItem, P1::OutputItem);

    fn next(&mut self, item: (P0::InputItem, P1::InputItem)) -> (P0::OutputItem, P1::OutputItem) {
        (self.0.next(item.0), self.1.next(item.1))
    }
}

pub trait Pipeline: Pipe<InputItem = (), OutputItem = bool> {
    fn run(&mut self) {
        while self.next(()) {
            // do nothing
        }
    }
}

impl<P> Pipeline for P where P: Pipe<InputItem = (), OutputItem = bool> {}

pub struct PipeIter<I: Iterator> {
    iter: I,
}

impl<I: Iterator> PipeIter<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> Pipe for PipeIter<I> {
    type InputItem = ();
    type OutputItem = Option<I::Item>;

    fn next(&mut self, _: ()) -> Option<I::Item> {
        self.iter.next()
    }
}

pub struct IterPipe<P: Pipe<InputItem = ()>> {
    pipe: P,
}

impl<P: Pipe<InputItem = ()>> IterPipe<P> {
    pub fn new(pipe: P) -> Self {
        Self { pipe }
    }
}

impl<O, P: Pipe<InputItem = (), OutputItem = Option<O>>> Iterator for IterPipe<P> {
    type Item = O;

    fn next(&mut self) -> Option<O> {
        self.pipe.next(())
    }
}
