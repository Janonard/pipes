use crate::{Connector, Pipe};
use std::ops::Shr;

/// A composable or composed pipe.
///
/// This struct is an implementation of the [newtype pattern](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types) to implement the `>>` operator for pipes (manifested as the `Shr` trait).
///
/// For more information, please see [the documentation of the `compose` method](trait.Pipe.html#method.compose).
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
    /// Create new composable or composed pipe.
    pub fn new(pipe: P) -> Self {
        Composed { pipe }
    }

    /// Unwrap the inner pipe.
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
