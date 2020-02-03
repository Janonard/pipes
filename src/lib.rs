pub mod util;

pub trait Pipe: Sized {
    type InputItem;
    type OutputItem;

    fn next(&mut self, item: Self::InputItem) -> Option<Self::OutputItem>;

    fn pre_map<I, F>(self, mapper: F) -> util::PreMap<I, Self, F>
    where
        F: Fn(I) -> Self::InputItem,
    {
        util::PreMap::new(self, mapper)
    }

    fn post_map<O, F>(self, mapper: F) -> util::PostMap<O, Self, F>
    where
        F: Fn(Self::OutputItem) -> O,
    {
        util::PostMap::new(self, mapper)
    }

    fn pre_connect<P>(self, pre_pipe: P) -> util::PipeConnector<P, Self>
    where
        P: Pipe<OutputItem = Self::InputItem>,
    {
        util::PipeConnector::new(pre_pipe, self)
    }

    fn post_connect<P>(self, post_pipe: P) -> util::PipeConnector<Self, P>
    where
        P: Pipe<InputItem = Self::OutputItem>,
    {
        util::PipeConnector::new(self, post_pipe)
    }
}

impl<P0, P1> Pipe for (P0, P1)
where
    P0: Pipe,
    P1: Pipe,
{
    type InputItem = (P0::InputItem, P1::InputItem);
    type OutputItem = (P0::OutputItem, P1::OutputItem);

    fn next(
        &mut self,
        item: (P0::InputItem, P1::InputItem),
    ) -> Option<(P0::OutputItem, P1::OutputItem)> {
        let item0 = self.0.next(item.0)?;
        let item1 = self.1.next(item.1)?;
        Some((item0, item1))
    }
}

pub trait Pipeline: Pipe<InputItem = (), OutputItem = ()> {
    fn run(&mut self) {
        loop {
            match self.next(()) {
                None => break,
                _ => (),
            }
        }
    }
}

impl<P> Pipeline for P where P: Pipe<InputItem = (), OutputItem = ()> {}

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
    type OutputItem = I::Item;

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

impl<P: Pipe<InputItem = ()>> Iterator for IterPipe<P> {
    type Item = P::OutputItem;

    fn next(&mut self) -> Option<P::OutputItem> {
        self.pipe.next(())
    }
}
