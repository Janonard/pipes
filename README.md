# Pipes-style stream processing

[![Build Status][travis-badge]][travis-url] [![Current Crates.io Version][crates-badge]][crates-url]

[travis-badge]: https://travis-ci.org/Janonard/pipes.svg?branch=master
[travis-url]: https://travis-ci.org/Janonard/pipes
[crates-badge]: https://img.shields.io/crates/v/pipes.svg
[crates-url]: https://crates.io/crates/pipes

This Rust crate contains an abstraction layer for compositional processing pipelines, inspired by Rust's [`Iterator`](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html) and Haskell's [`pipes` library](https://hackage.haskell.org/package/pipes).

The heart of the crate is the `Pipe` trait, which basically boils down to:
``` rs
trait Pipe {
    type InputItem;
    type OutputItem;

    fn next(&mut self, input: Self::InputItem) -> Self::OutputItem;
}
```
It is similar to `Iterator` as it produces a stream of items, but it does so by also consuming a stream of items. Just like `Iterator`, it also has a lot of decoration methods for common modifications to pipes and they can also be concatenated to form bigger pipes.

## An example

This is an example on how someone would design a square wave generator:

The pipeline is split into several parts. First, it takes an open range iterator and wraps it in a pipe. You can do that since iterators can be seen as a pipe that consumes an `()` and produces an `Option<T>` of some arbitrary `T`. Then, it uses a lazily constructed piece of pipe that unwraps the value.

Now, we have to turn that stream of indices into some sort of wave. This is done by two custom pipes: `Progress` and `SquareWave`. The `Progress` pipe accepts a stream of indices and wraps them by a given wave length. It also divides the wrapped indices by the length of the wave, which basically creates a sawtooth wave ranging from `0.0` to `1.0`.

This signal can be used to render any waveform independently of the wave length, which the `SquareWave` pipe does.

``` rs
use iterpipes::*;

/// A pipe that turns an index into a periodic progress value between 0.0 and 1.0.
struct Progress {
    period_length: usize,
}

impl Pipe for Progress {
    type InputItem = usize;
    type OutputItem = f32;

    fn next(&mut self, index: usize) -> f32 {
        (index % self.period_length) as f32 / self.period_length as f32
    }
}

/// A pipe that turns a progress value into a square wave.
struct SquareWave;

impl Pipe for SquareWave {
    type InputItem = f32;
    type OutputItem = f32;

    fn next(&mut self, progress: f32) -> f32 {
        if progress < 0.5 {
            -1.0
        } else {
            1.0
        }
    }
}

// Putting it all together
let mut pipe = PipeIter::new(0..).compose()
    >> Lazy::new(|i: Option<usize>| i.unwrap())
    >> Progress {period_length: 4}.compose()
    >> SquareWave;

// Asserting that it works!
for frame in &[-1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0] {
    assert_eq!(*frame, pipe.next(()));
}
```

## QnA

### Why should I use `Pipe`?

The alternative to `Pipe` is to write it all as one big function. However, this makes it hard to test the individual parts of the algorithm and to re-use it for different tasks. Using well-defined and granular pipes however boosts the reusability and the testability of your code.

In the example above, you could for example swap out the square wave generator with a sine wave generator or a sample mapper without needing to worry about the other stuff. It would also be very easy use the same code to play longer samples and slow them down while they play.

Another advantage of having many small methods is that you can write individual unit tests for them. This can be especially handy when you're working with real-time audio processing and you can't debug your program while it is running.

### Is it slower than writing it out manually?

Using many small functions to build one big one obviously introduces an overhead. However, this overhead is only marginal, about 1-2%, and can also be removed completely by enabling link-time optimizations. When enabled, the llvm linker evaluates the final binary (program, shared object or static library) as whole and optimizes and inlines across function and crate boundaries. Simply add the following lines to your `Cargo.toml`:

``` toml
[profile.release]
lto = true

[profile.bench]
lto = true
```

This project also contains a small benchmark that calculates the signal of a metronome using a sine wave and an attack-decay envelope. Depending on your machine, it will take about 15 minutes.

With lto turned off, the `Pipe`-based implementation is about 1-2% slower than a manual implementation, but with lto turned on, they're exactly the same.

### Can I use it for my commercial products?

Yes, you can! You can license the code either using the MIT or the Apache 2.0 license, which means that you can do almost anything you want!