use crate::{Connector, Pipe};
use std::ops::Shr;

pub struct Composed<P>
where
    P: Pipe,
{
    pipe: P,
}

impl<P> Composed<P>
where
    P: Pipe,
{
    pub fn new(pipe: P) -> Self {
        Composed { pipe }
    }

    pub fn unwrap(self) -> P {
        self.pipe
    }
}

impl<P> Pipe for Composed<P>
where
    P: Pipe,
{
    type InputItem = P::InputItem;
    type OutputItem = P::OutputItem;

    #[inline]
    fn next(&mut self, item: P::InputItem) -> P::OutputItem {
        self.pipe.next(item)
    }
}

impl<P0, P1> Shr<P1> for Composed<P0>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    type Output = Composed<Connector<P0, P1>>;

    fn shr(self, other: P1) -> Self::Output {
        Self::Output::new(Connector::new(self.pipe, other))
    }
}
