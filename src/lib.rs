pub trait Pipe {
    type InputItem;
    type OutputItem;

    fn next(&mut self, item: Self::InputItem) -> Self::OutputItem;

    fn bypass(self) -> Bypass<Self>
    where
        Self: Sized,
        Self::InputItem: Clone,
    {
        Bypass::new(self)
    }

    fn compose(self) -> Compose<Self::InputItem, Self::OutputItem, Self>
    where
        Self: Sized,
    {
        Compose::new(self)
    }

    fn connect<O: Pipe<InputItem = Self::OutputItem>>(self, other: O) -> Connector<Self, O>
    where
        Self: Sized,
    {
        Connector::new(self, other)
    }

    fn into_iter(self) -> IterPipe<Self>
    where
        Self: Sized + Pipe<InputItem = ()>,
    {
        IterPipe::new(self)
    }

    fn optional(self) -> OptionMap<Self>
    where
        Self: Sized,
    {
        OptionMap::new(self)
    }
}

mod util;
pub use util::*;

mod iter;
pub use iter::*;

mod compose;
pub use compose::*;

#[cfg(test)]
mod tests;
