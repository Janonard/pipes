//! Compositional, pipes-style stream processing.
//!
//! This crate contains an abstraction layer for compositional processing pipelines, inspired by Rust's [`Iterator`](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html) and Haskell's [`pipes` library](https://hackage.haskell.org/package/pipes).
//!
//! The heart of this crate is the [`Pipe` trait](trait.Pipe.html). It has an input item type and an output item type, as well as the [`next` method](trait.Pipe.html#tymethod.next) to calculate the next output item from the next input item. Everything else is built upon this concept.
//!
//! The two main advantages of using `Pipe` over implementing pipelines "manually" is that granular pieces of pipe can be tested individually and combined into larger pipes; They improve the testability and reusability of the code.
//!
//! # Implementing `Pipe`
//!
//! Implementing `Pipe` is similar to implementing `Iterator`, but even more simple. Let's create pipe that multiplies every input item with a previously set factor:
//!
//! ```
//! use iterpipes::Pipe;
//! use std::ops::Mul;
//!
//! struct Multiply<V: Mul + Copy> {
//!     factor: V,
//! }
//!
//! impl<V: Mul + Copy> Pipe for Multiply<V> {
//!     type InputItem = V;
//!     type OutputItem = V::Output;
//!
//!     fn next(&mut self, input: V) -> V::Output {
//!         input * self.factor
//!     }
//! }
//!
//! let mut multiply: Multiply<u32> = Multiply { factor: 2 };
//!
//! assert_eq!(4, multiply.next(2));
//! assert_eq!(8, multiply.next(4));
//! ```
//!
//! # Decoration and Composition
//!
//! Once the individual and granular pipes are implemented and tested, they can be decorated and combined into big and complex pipelines. First, `Pipe` has many decorator methods, just like `Iterator`, which create a new pipe with new behavior that is based on the old one.
//!
//! Secondly, you can compose them using the `>>` operator. Prior to this, you have to turn the first pipe of the composition into a composable one using the [`compose` method](trait.Pipe.html#method.compose). Then, you can connect fitting pipes together into a big one.
//!
//! Let's reuse the `Multiply` pipe from above and apply it to a pulse wave generator:
//!
//! ```
//! use iterpipes::Pipe;
//! use std::ops::Mul;
//!
//! /// A pipe that multiplies any signal by a given factor.
//! struct Multiply<V: Mul + Copy> {
//!     factor: V,
//! }
//!
//! impl<V: Mul + Copy> Pipe for Multiply<V> {
//!     type InputItem = V;
//!     type OutputItem = V::Output;
//!
//!     #[inline]
//!     fn next(&mut self, input: V) -> V::Output {
//!         input * self.factor
//!     }
//! }
//!
//! /// A pipe that generates a square wave from a given index.
//! struct PulseWave {
//!     pulse_length: usize,
//! }
//!
//! impl Pipe for PulseWave {
//!     type InputItem = usize;
//!     type OutputItem = f32;
//!
//!     #[inline]
//!     fn next(&mut self, index: usize) -> f32 {
//!         // If the index is part of an even pulse, return 1.0 and -1.0 otherwise.
//!         if (index / self.pulse_length) % 2 == 0 {
//!             1.0
//!         } else {
//!             -1.0
//!         }
//!     }
//! }
//!
//! // Compose the two pipes into one.
//! let mut combined = PulseWave { pulse_length: 2 }.compose() >> Multiply { factor: 0.5 };
//!
//! for i in 0..32 {
//!     let frame = combined.next(i);
//!     if (i / 2) % 2 == 0 {
//!         assert_eq!(frame, 0.5);
//!     } else {
//!         assert_eq!(frame, -0.5);
//!     }
//! }
//! ```
//!
//! # Interoperability
//!
//! There are interoperability layers to use a `Pipe` as an `Iterator` and vice-versa. These are [`IterPipe`](struct.IterPipe.html) and [`PipeIter`](struct.PipeIter.html).
//!
//! Let's have an example that iterates over a slice, multiplies every value by two and collects it into a vector:
//!
//! ```
//! use iterpipes::{Pipe, Lazy, PipeIter};
//! use std::ops::Mul;
//!
//! /// A pipe that multiplies any signal by a given factor.
//! struct Multiply<V: Mul + Copy> {
//!     factor: V,
//! }
//!
//! impl<V: Mul + Copy> Pipe for Multiply<V> {
//!     type InputItem = V;
//!     type OutputItem = V::Output;
//!
//!     #[inline]
//!     fn next(&mut self, input: V) -> V::Output {
//!         input * self.factor
//!     }
//! }
//!
//! let input: Vec<usize> = (0..16).collect();
//!
//! // Create an iterator over the input.
//! let pipeline = input.iter().cloned();
//! // Turn it into a pipe.
//! let pipeline = PipeIter::new(pipeline).compose();
//! // Connect it to an optional version of the multiplication pipe.
//! let pipeline = pipeline >> Multiply { factor: 2}.optional();
//! // Turn the pipe back to an iterator.
//! let pipeline = pipeline.into_iter();
//!
//! // Collect and verify the results.
//! let result: Vec<usize> = pipeline.collect();
//! for i in 0..16 {
//!     assert_eq!(result[i], i*2);
//! }
//! ```
//!
//! # A note on performance
//!
//! Using pipes to express processing streams has side-effects on the performance. Since the resulting algorithm is created from many small functions instead of one big one, there is an overhead when these functions are called. It might also be harder for the compiler to use [SIMD instructions](https://en.wikipedia.org/wiki/SIMD).
//!
//! These effects are removed when the resulting binary (program, shared object or static library) is compiled with link-time optimizations turned on. This will lead to the linker evaluating the compiled program as a whole and optimizing and inlining across functions and even crates.
//!
//! These can be enabled by adding the following lines to your `Cargo.toml`:
//!
//! ``` toml
//! [profile.release]
//! lto = true
//!
//! [profile.bench]
//! lto = true
//! ```

/// An iterator-style pipe.
///
/// For more general information about pipes, please see the [module-level documentation](index.html).
pub trait Pipe {
    /// The type of input this pipe accepts.
    type InputItem;
    /// The type of output this pipe produces.
    type OutputItem;

    /// Calculate the next output item, based on an input item.
    fn next(&mut self, item: Self::InputItem) -> Self::OutputItem;

    /// Reset the state of the pipe.
    ///
    /// If implemented, this method resets the pipe to the state it had before the first output was retrieved. Since all decorator pipes of this crate implement it, it can be be used to reset the state of a whole pipeline without needing to constructing it again.
    ///
    /// If your pipe can't be reseted, you may use the `unimplemented!()` macro. However, you should note this behavior in your documentation!
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that counts up.
    /// struct Counter {
    ///     index: usize,
    /// }
    ///
    /// impl Pipe for Counter {
    ///     type InputItem = ();
    ///     type OutputItem = usize;
    ///
    ///     fn next(&mut self, _: ()) -> usize {
    ///         let output = self.index;
    ///         self.index += 1;
    ///         output
    ///     }
    ///
    ///     fn reset(&mut self) {
    ///         self.index = 0;
    ///     }
    /// }
    ///
    /// let mut counter = Counter { index: 0};
    /// assert_eq!(0, counter.next(()));
    /// assert_eq!(1, counter.next(()));
    /// counter.reset();
    /// assert_eq!(0, counter.next(()));
    /// assert_eq!(1, counter.next(()));
    /// ```
    fn reset(&mut self) {}

    /// Create a bypassed version of the pipe.
    ///
    /// The returned pipe clones the input item, calculates the next output item and returns both
    /// the copied input item and the output item.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that rounds a floating point value to the nearest integer.
    /// struct Round;
    ///
    /// impl Pipe for Round {
    ///     type InputItem = f32;
    ///     type OutputItem = i32;
    ///
    ///     fn next(&mut self, input: f32) -> i32 {
    ///         input.round() as i32
    ///     }
    /// }
    ///
    /// let mut pipe = Round {}.bypass();
    /// assert_eq!((0.5, 1), pipe.next(0.5));
    /// assert_eq!((-2.2, -2), pipe.next(-2.2));
    /// ```
    fn bypass(self) -> Bypass<Self>
    where
        Self: Sized,
        Self::InputItem: Clone,
    {
        Bypass::new(self)
    }

    /// Create a composable pipe.
    ///
    /// Composable pipes implement the `>>` operator that concatenates pipes.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that turns an index into a periodic progress value between 0.0 and 1.0.
    /// struct Progress {
    ///     period_length: usize,
    /// }
    ///
    /// impl Pipe for Progress {
    ///     type InputItem = usize;
    ///     type OutputItem = f32;
    ///
    ///     fn next(&mut self, index: usize) -> f32 {
    ///         (index % self.period_length) as f32 / self.period_length as f32
    ///     }
    /// }
    ///
    /// /// A pipe that turns a progress value into a square wave.
    /// struct SquareWave;
    ///
    /// impl Pipe for SquareWave {
    ///     type InputItem = f32;
    ///     type OutputItem = f32;
    ///
    ///     fn next(&mut self, progress: f32) -> f32 {
    ///         if progress < 0.5 {
    ///             -1.0
    ///         } else {
    ///             1.0
    ///         }
    ///     }
    /// }
    ///
    /// let mut pipe = PipeIter::new(0..).compose()
    ///     >> Lazy::new(|i: Option<usize>| i.unwrap())
    ///     >> Progress {period_length: 4}.compose()
    ///     >> SquareWave;
    ///
    /// for frame in &[-1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0] {
    ///     assert_eq!(*frame, pipe.next(()));
    /// }
    /// ```
    ///
    /// # A technical note
    ///
    /// The `Compose` struct is a workaround the fact that this crate can not implement the `Shr` trait (the `>>` operator) for every type that implements `Pipe` since `Shr` isn't a part of this crate. This patttern is known as [the newtype pattern](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-the-newtype-pattern-to-implement-external-traits-on-external-types).
    fn compose(self) -> Composed<Self>
    where
        Self: Sized,
    {
        Composed::new(self)
    }

    /// Connect two pipes.
    ///
    /// The created pipe takes an input item for `self`, calculates the intermediate value and then uses it to calculate the output value of the `other` pipe.
    ///
    /// Obviously, the `InputItem` of `self` and the `OutputItem` of the `other` pipe have to match!
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that turns an index into a periodic progress value between 0.0 and 1.0.
    /// struct Progress {
    ///     period_length: usize,
    /// }
    ///
    /// impl Pipe for Progress {
    ///     type InputItem = usize;
    ///     type OutputItem = f32;
    ///
    ///     fn next(&mut self, index: usize) -> f32 {
    ///         (index % self.period_length) as f32 / self.period_length as f32
    ///     }
    /// }
    ///
    /// /// A pipe that turns a progress value into a square wave.
    /// struct SquareWave;
    ///
    /// impl Pipe for SquareWave {
    ///     type InputItem = f32;
    ///     type OutputItem = f32;
    ///
    ///     fn next(&mut self, progress: f32) -> f32 {
    ///         if progress < 0.5 {
    ///             -1.0
    ///         } else {
    ///             1.0
    ///         }
    ///     }
    /// }
    ///
    /// let mut pipe = Progress {period_length: 4}.connect(SquareWave);
    ///
    /// for (index, frame) in [-1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0].iter().enumerate() {
    ///     assert_eq!(*frame, pipe.next(index));
    /// }
    /// ```
    fn connect<O: Pipe<InputItem = Self::OutputItem>>(self, other: O) -> Connector<Self, O>
    where
        Self: Sized,
    {
        Connector::new(self, other)
    }

    /// Wrap the pipe into an iterator.
    ///
    /// For example, this can be used to iterate over a pipeline in a `for` loop. The input item needs to have a default value, since the iterator has to create it on it's own, and the output item must be an `Option`al value.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// An pipe/iterator over a slice.
    /// struct SlicePipe<'a, T> {
    ///     data: &'a [T],
    ///     index: usize,
    /// }
    ///
    /// impl<'a, T> Pipe for SlicePipe<'a, T> {
    ///     type InputItem = ();
    ///     type OutputItem = Option<&'a T>;
    ///
    ///     fn next(&mut self, _: ()) -> Option<&'a T> {
    ///         let value = self.data.get(self.index);
    ///         if value.is_some() {
    ///             self.index += 1;
    ///         }
    ///         value
    ///     }
    /// }
    ///
    /// const DATA: &[u32] = &[3, 2, 1];
    /// for (index, value) in (SlicePipe {data: DATA, index: 0}).into_iter().enumerate() {
    ///     assert_eq!(DATA[index], *value);
    /// }
    /// ```
    fn into_iter(self) -> IterPipe<Self>
    where
        Self::InputItem: Default,
        Self: Sized + Pipe<InputItem = ()>,
    {
        IterPipe::new(self)
    }

    /// Optionalize the pipe.
    ///
    /// The decorated pipe's input and output items are the optional versions of the original input and output items. If an input item is fed into the decorated pipe, it returns some output value, but if `None` is fed into the decorated pipe, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that multiplies an input item by a factor.
    /// struct Multiply<T>
    /// where
    ///     T: std::ops::Mul<T> + Copy
    /// {
    ///     factor: T,
    /// }
    ///
    /// impl<T> Pipe for Multiply<T>
    /// where
    ///     T: std::ops::Mul<T> + Copy
    /// {
    ///     type InputItem = T;
    ///     type OutputItem = T::Output;
    ///
    ///     fn next(&mut self, item: T) -> T::Output {
    ///         item * self.factor
    ///     }
    /// }
    ///
    /// let mut pipe = Multiply::<u32> { factor: 2 }.optional();
    ///
    /// assert_eq!(Some(4), pipe.next(Some(2)));
    /// assert_eq!(None, pipe.next(None));
    /// ```
    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        Optional::new(self)
    }

    /// Enumerate the output items of a pipe.
    ///
    /// The decorated pipe will return a tuple of an index and the output item. The index starts from 0 and is counted up for every output item.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// /// A pipe that always returns a clone of the same value.
    /// struct DefaultPipe<T: Clone> {
    ///     value: T,
    /// }
    ///
    /// impl<T: Clone> Pipe for DefaultPipe<T> {
    ///     type InputItem = ();
    ///     type OutputItem = T;
    ///
    ///     fn next(&mut self, _: ()) -> T {
    ///         self.value.clone()
    ///     }
    /// }
    ///
    /// let mut pipe = DefaultPipe { value: 42u8 }.enumerate();
    /// assert_eq!((0, 42), pipe.next(()));
    /// assert_eq!((1, 42), pipe.next(()));
    /// assert_eq!((2, 42), pipe.next(()));
    /// ```
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }

    /// Create a boxed trait object of the pipe.
    ///
    /// This might be useful to move pipes across API bounds since it hides the internal composition of the pipe.
    ///
    /// # Example
    ///
    /// ```
    /// use iterpipes::*;
    ///
    /// fn create_pipe() -> Box<dyn Pipe<InputItem = usize, OutputItem = usize>> {
    ///     Lazy::new(|i| i * 2).boxed()
    /// }
    ///
    /// let mut pipe = create_pipe();
    ///
    /// for i in 0..4 {
    ///     assert_eq!(i*2, pipe.next(i));
    /// }
    /// ```
    fn boxed(self) -> Box<dyn Pipe<InputItem = Self::InputItem, OutputItem = Self::OutputItem>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl Pipe for () {
    type InputItem = ();
    type OutputItem = ();

    fn next(&mut self, _: ()) {}

    fn reset(&mut self) {}
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
        (p0_input, p1_input): (P0::InputItem, P1::InputItem),
    ) -> (P0::OutputItem, P1::OutputItem) {
        (self.0.next(p0_input), self.1.next(p1_input))
    }

    fn reset(&mut self) {
        self.0.reset();
        self.1.reset();
    }
}

impl<'a, P: Pipe + ?Sized> Pipe for &'a mut P {
    type InputItem = P::InputItem;
    type OutputItem = P::OutputItem;

    fn next(&mut self, input: P::InputItem) -> P::OutputItem {
        (*self).next(input)
    }

    fn reset(&mut self) {
        (*self).reset();
    }
}

mod util;
pub use util::*;

mod iter;
pub use iter::*;

mod composed;
pub use composed::*;

#[test]
fn trait_object() {
    let mut pipe: Box<dyn Pipe<InputItem = (), OutputItem = Option<usize>>> =
        PipeIter::new((0..42).map(|_| 42)).boxed();

    while let Some(i) = pipe.next(()) {
        assert_eq!(i, 42);
    }
}
