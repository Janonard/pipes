//! Pipes-style stream processing.
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
//! use iterpipe::Pipe;
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
//!     #[inline]
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
//! The `#[inline]` attribute is important: Most of the time, pipes are used within other pipes and their `next` method is only called in one place. Therefore, they are a good candidate for inlining. Not inlining the `next` methods may also have bad side effects. For example, if the `next` methods aren't inlined, the compiler won't be able to use [SIMD instructions](https://en.wikipedia.org/wiki/SIMD), which will result in a big performance loss.
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
//! use iterpipe::Pipe;
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
//! use iterpipe::{Pipe, Lazy, PipeIter};
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

/// An iterator-style pipe.
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
        Self::InputItem: Default,
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

    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate::new(self)
    }

    fn boxed(self) -> Box<dyn Pipe<InputItem = Self::InputItem, OutputItem = Self::OutputItem>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }

    fn boxed_send(
        self,
    ) -> Box<dyn Send + Pipe<InputItem = Self::InputItem, OutputItem = Self::OutputItem>>
    where
        Self: Sized + Send + 'static,
    {
        Box::new(self)
    }

    fn boxed_sync(
        self,
    ) -> Box<dyn Sync + Pipe<InputItem = Self::InputItem, OutputItem = Self::OutputItem>>
    where
        Self: Sized + Sync + 'static,
    {
        Box::new(self)
    }

    fn boxed_send_sync(
        self,
    ) -> Box<dyn Send + Sync + Pipe<InputItem = Self::InputItem, OutputItem = Self::OutputItem>>
    where
        Self: Sized + Send + Sync + 'static,
    {
        Box::new(self)
    }
}

impl Pipe for () {
    type InputItem = ();
    type OutputItem = ();

    #[inline]
    fn next(&mut self, _: ()) -> () {}
}

mod util;
pub use util::*;

mod iter;
pub use iter::*;

mod compose;
pub use compose::*;

#[cfg(test)]
mod tests;
