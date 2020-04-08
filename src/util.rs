use crate::Pipe;
use std::marker::PhantomData;

/// A pipe that connects two other pipes together.
///
/// The input item of this connector is the input item of `P0` and its output item is the output item of `P1`. It calculates the output item of `P0` and feeds it directly into `P1`.
///
/// Obviously, the output item of `P0` has to match the input item of `P1`.
///
/// For more information, please see [the documentation of the `connect` method](trait.Pipe.html#method.connect).
pub struct Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    pipe0: P0,
    pipe1: P1,
}

impl<P0, P1> Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    /// Create a new connector with the two pipes.
    pub fn new(pipe0: P0, pipe1: P1) -> Self {
        Connector { pipe0, pipe1 }
    }
}

impl<P0, P1> Pipe for Connector<P0, P1>
where
    P0: Pipe,
    P1: Pipe<InputItem = P0::OutputItem>,
{
    type InputItem = P0::InputItem;
    type OutputItem = P1::OutputItem;

    fn next(&mut self, input: Self::InputItem) -> Self::OutputItem {
        self.pipe1.next(self.pipe0.next(input))
    }
}

/// A pipe that bypasses the effects of an internal pipe.
///
/// For more information, please see [the documentation of the `bypass` method](trait.Pipe.html#method.bypass).
pub struct Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    pipe: P,
}

impl<P> Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    /// Create a new bypassed pipe.
    pub fn new(pipe: P) -> Self {
        Self { pipe }
    }
}

impl<P> Pipe for Bypass<P>
where
    P: Pipe,
    P::InputItem: Clone,
{
    type InputItem = P::InputItem;
    type OutputItem = (P::InputItem, P::OutputItem);

    fn next(&mut self, input: P::InputItem) -> (P::InputItem, P::OutputItem) {
        (input.clone(), self.pipe.next(input))
    }
}

/// A "lazily" create pipe.
///
/// This pipe's behavior is defined by a callable object, for example a lambda expression, and can therefore be "lazily" created inline.
///
/// # Example
///
/// ```
/// use iterpipes::*;
///
/// let mut pipe = Lazy::new(|i: u8| 2*i);
/// assert_eq!(2, pipe.next(1));
/// assert_eq!(4, pipe.next(2));
/// ```
pub struct Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    function: F,
    input: PhantomData<I>,
    output: PhantomData<O>,
}

impl<I, O, F> Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    /// Create a new lazy pipe.
    pub fn new(function: F) -> Self {
        Self {
            function,
            input: PhantomData,
            output: PhantomData,
        }
    }
}

impl<I, O, F> Pipe for Lazy<I, O, F>
where
    F: FnMut(I) -> O,
{
    type InputItem = I;
    type OutputItem = O;

    fn next(&mut self, input: I) -> O {
        (self.function)(input)
    }
}

/// A pipe that wraps another pipe's IO in an `Option`.
///
/// For more information, please see [the documentation of the `optional` method](trait.Pipe.html#method.optional).
pub struct Optional<P>
where
    P: Pipe,
{
    pipe: P,
}

impl<P: Pipe> Optional<P> {
    /// Create a new optional pipe.
    pub fn new(pipe: P) -> Self {
        Optional { pipe }
    }
}

impl<P> Pipe for Optional<P>
where
    P: Pipe,
{
    type InputItem = Option<P::InputItem>;
    type OutputItem = Option<P::OutputItem>;

    fn next(&mut self, item: Option<P::InputItem>) -> Option<P::OutputItem> {
        item.map(|item| self.pipe.next(item))
    }
}

/// A pipe that enumerates the output items of another pipe.
///
/// The inputs of this pipe are the same as the wrapped ones, but it's output item is a tuple of an index and the wrapped pipe's output. The index starts with zero and counts up for every produces output item.
///
/// For more information, please see [the documentation of the `enumerate` method](trait.Pipe.html#method.enumerate).
pub struct Enumerate<P>
where
    P: Pipe,
{
    pipe: P,
    progress: usize,
}

impl<P: Pipe> Enumerate<P> {
    /// Create a new enumerating pipe.
    pub fn new(pipe: P) -> Self {
        Enumerate { pipe, progress: 0 }
    }
}

impl<P: Pipe> Pipe for Enumerate<P> {
    type InputItem = P::InputItem;
    type OutputItem = (usize, P::OutputItem);

    fn next(&mut self, item: P::InputItem) -> (usize, P::OutputItem) {
        let next_item = self.pipe.next(item);
        let index = self.progress;
        self.progress += 1;
        (index, next_item)
    }
}
