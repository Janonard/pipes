use crate::{Connector, Pipe};
use std::marker::PhantomData;
use std::ops::Shr;

pub struct Compose<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    pipe: P,
    item: PhantomData<(I, O)>,
}

impl<I, O, P> Compose<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    pub fn new(pipe: P) -> Self {
        Compose {
            pipe,
            item: PhantomData,
        }
    }
}

impl<I, O, P> Pipe for Compose<I, O, P>
where
    P: Pipe<InputItem = I, OutputItem = O>,
{
    type InputItem = I;
    type OutputItem = O;

    #[inline]
    fn next(&mut self, item: I) -> O {
        self.pipe.next(item)
    }
}

impl<I, M, P0, P1> Shr<P1> for Compose<I, M, P0>
where
    P0: Pipe<InputItem = I, OutputItem = M>,
    P1: Pipe<InputItem = M>,
{
    type Output = Compose<P0::InputItem, P1::OutputItem, Connector<P0, P1>>;

    fn shr(self, other: P1) -> Self::Output {
        Self::Output::new(Connector::new(self.pipe, other))
    }
}
