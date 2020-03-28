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
