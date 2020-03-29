use pipes::*;
use time::{Duration, Instant};

mod piped {
    use pipes::Pipe;

    struct Envelope {
        attack_len: usize,
        decay_len: usize,
    }

    impl Envelope {
        pub fn new(attack_len: usize, decay_len: usize) -> Self {
            Self {
                attack_len,
                decay_len,
            }
        }
    }

    impl Pipe for Envelope {
        type InputItem = usize;
        type OutputItem = f32;

        #[inline]
        fn next(&mut self, index: usize) -> f32 {
            if index < self.attack_len {
                index as f32 / self.attack_len as f32
            } else if index < self.attack_len + self.decay_len {
                1.0 - ((index - self.attack_len) as f32 / self.decay_len as f32)
            } else {
                0.0
            }
        }
    }

    struct SineWave {
        wave_length: usize,
    }

    impl SineWave {
        pub fn new(wave_length: usize) -> Self {
            SineWave { wave_length }
        }
    }

    impl Pipe for SineWave {
        type InputItem = usize;
        type OutputItem = f32;

        #[inline]
        fn next(&mut self, index: usize) -> f32 {
            let index = index % self.wave_length;
            let progress = index as f32 / self.wave_length as f32;
            (progress * 2.0 * std::f32::consts::PI).sin()
        }
    }

    pub struct Metronome {
        env: Envelope,
        sine: SineWave,
        pulse_distance: usize,
    }

    impl Metronome {
        pub fn new(
            attack_len: usize,
            decay_len: usize,
            wave_length: usize,
            pulse_distance: usize,
        ) -> Self {
            Self {
                env: Envelope::new(attack_len, decay_len),
                sine: SineWave::new(wave_length),
                pulse_distance,
            }
        }
    }

    impl Pipe for Metronome {
        type InputItem = usize;
        type OutputItem = f32;

        fn next(&mut self, index: usize) -> f32 {
            self.sine.next(index) * self.env.next(index % self.pulse_distance)
        }
    }
}

mod manual {
    use pipes::Pipe;

    pub struct Metronome {
        attack_len: usize,
        decay_len: usize,
        wave_length: usize,
        pulse_distance: usize,
    }

    impl Metronome {
        pub fn new(
            attack_len: usize,
            decay_len: usize,
            wave_length: usize,
            pulse_distance: usize,
        ) -> Self {
            Self {
                attack_len,
                decay_len,
                wave_length,
                pulse_distance,
            }
        }
    }

    impl Pipe for Metronome {
        type InputItem = usize;
        type OutputItem = f32;

        fn next(&mut self, index: usize) -> f32 {
            let wave_index = index % self.wave_length;
            let wave_progress = wave_index as f32 / self.wave_length as f32;
            let wave_frame = (wave_progress * 2.0 * std::f32::consts::PI).sin();

            let env_index = index % self.pulse_distance;
            let env_frame = if env_index < self.attack_len {
                index as f32 / self.attack_len as f32
            } else if env_index < self.attack_len + self.decay_len {
                1.0 - ((env_index - self.attack_len) as f32 / self.decay_len as f32)
            } else {
                0.0
            };

            wave_frame * env_frame
        }
    }
}

fn benchmark_pipe<P, F>(length: usize, runs: usize, mut factory: F)
where
    P: Pipe<InputItem = usize, OutputItem = f32>,
    F: FnMut() -> P,
{
    let mut signal_buffer = vec![0.0; length].into_boxed_slice();
    let mut durations: Vec<Duration> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let mut pipe = factory();
        let start = Instant::now();
        for i in 0..length {
            signal_buffer[i] = pipe.next(i);
        }
        let end = Instant::now();
        let duration = end - start;
        print!("{}, ", duration.as_seconds_f32());
        durations.push(duration);
    }
    println!();

    let durations: Vec<f32> = durations
        .iter()
        .map(|duration| duration.as_seconds_f32())
        .collect();
    println!(
        "# Mean: {}",
        durations.iter().cloned().sum::<f32>() / runs as f32
    );
}

fn main() {
    const LEN: usize = 200_000_000;
    const RUNS: usize = 10;

    println!("# This program benchmarks a simple metronome,");
    println!("# once implemented with pipes and once implemented manually.");
    println!("# The individual runtimes are printed as CSV.");

    println!("# Runtimes of the piped version:");
    let mut counter: usize = 0;
    benchmark_pipe(LEN, RUNS, move || {
        counter += 1;
        piped::Metronome::new(500, 500, 100 * counter, 1_000)
    });

    println!("# Runtimes of the manual version:");
    let mut counter: usize = 0;
    benchmark_pipe(LEN, RUNS, move || {
        counter += 1;
        manual::Metronome::new(500, 500, 100 * counter, 1_000)
    });
}
