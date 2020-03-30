use crate::Pipe;

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

    #[inline]
    fn next(&mut self, _: ()) -> Option<I::Item> {
        self.iter.next()
    }
}

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

    #[inline]
    fn next(&mut self) -> Option<O> {
        self.pipe.next(P::InputItem::default())
    }
}
