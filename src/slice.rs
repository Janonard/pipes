use crate::Pipe;

pub struct SliceProducer<'a, T> {
    internal: crate::PipeIter<std::slice::Iter<'a, T>>,
}

impl<'a, T> SliceProducer<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            internal: crate::PipeIter::new(slice.iter()),
        }
    }
}

impl<'a, T> Pipe for SliceProducer<'a, T> {
    type InputItem = ();
    type OutputItem = Option<&'a T>;

    fn next(&mut self, _: ()) -> Option<&'a T> {
        self.internal.next(())
    }
}

pub struct SliceProducerMut<'a, T> {
    internal: crate::PipeIter<std::slice::IterMut<'a, T>>,
}

impl<'a, T> SliceProducerMut<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        Self {
            internal: crate::PipeIter::new(slice.iter_mut()),
        }
    }
}

impl<'a, T> Pipe for SliceProducerMut<'a, T> {
    type InputItem = ();
    type OutputItem = Option<&'a mut T>;

    fn next(&mut self, _: ()) -> Option<&'a mut T> {
        self.internal.next(())
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ConsumeResult {
    Ok,
    LastItem,
    Full,
}

pub struct SliceConsumer<'a, T> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T> SliceConsumer<'a, T> {
    pub fn new(slice: &'a mut [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T> Pipe for SliceConsumer<'a, T> {
    type InputItem = T;
    type OutputItem = ConsumeResult;

    fn next(&mut self, t: T) -> ConsumeResult {
        if self.index < self.slice.len() {
            self.slice[self.index] = t;
            self.index += 1;
            if self.index == self.slice.len() {
                ConsumeResult::LastItem
            } else {
                ConsumeResult::Ok
            }
        } else {
            ConsumeResult::Full
        }
    }
}
