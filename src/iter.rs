use crate::Pipe;

/// A pipe that yields the elements of an iterator.
///
/// As iterators don't have input items, this always takes a `()` and returns the next value of the iterators. Also, iterators can not be reseted and therefore, `reset` will panic if it's called.
pub struct PipeIter<I: Iterator> {
    iter: I,
}

impl<I: Iterator> PipeIter<I> {
    /// Create a new pipe wrapper for that iterator.
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

/// An iterator that yields values by creating a default value and running it through a pipe.
///
/// The input value for the pipe obviously must implement `Default` and the output item of the pipe must be an `Option<T>`.
pub struct IterPipe<P>
where
    P: Pipe,
    P::InputItem: Default,
{
    pipe: P,
}

impl<P> IterPipe<P>
where
    P: Pipe,
    P::InputItem: Default,
{
    /// Create a new iterator with that pipe.
    pub fn new(pipe: P) -> Self {
        Self { pipe }
    }
}

impl<O, P> Iterator for IterPipe<P>
where
    P: Pipe<OutputItem = Option<O>>,
    P::InputItem: Default,
{
    type Item = O;

    fn next(&mut self) -> Option<O> {
        self.pipe.next(P::InputItem::default())
    }
}
