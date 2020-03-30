use iterpipe::*;
use time::Instant;

mod piped {
    use iterpipe::Pipe;

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

        #[inline]
        fn next(&mut self, index: usize) -> f32 {
            self.sine.next(index) * self.env.next(index % self.pulse_distance)
        }
    }
}

mod manual {
    use iterpipe::Pipe;

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

        #[inline]
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

fn benchmark_pipe<P, F>(length: usize, runs: usize, mut factory: F) -> Vec<f32>
where
    P: Pipe<InputItem = usize, OutputItem = f32>,
    F: FnMut() -> P,
{
    let mut signal_buffer = vec![0.0; length].into_boxed_slice();
    let mut durations: Vec<f32> = Vec::with_capacity(runs);

    for _ in 0..runs {
        let mut pipe = factory();
        let start = Instant::now();
        for i in 0..length {
            signal_buffer[i] = pipe.next(i);
        }
        let end = Instant::now();
        let duration = (end - start).as_seconds_f32();
        print!("{}, ", duration);
        durations.push(duration);
    }
    println!();

    durations
}

const INFO: &str = "# This program benchmarks pipes by rendering a simple metronome signal.
# The signal is calculated by a pipes-based implementation first and by a manually implementated one
# afterwards. Both implementations are executed 200 times each, which will take about 15 minutes,
# depending on your system. The runtime of each execution is printed in a CSV-style format, which
# can parsed and analyzed.
#
# This benchmark shows that pipes-based implementations aren't significantly slower than manually
# implemented ones; Only by 1.0% ± 0.7%.
#
";

fn main() {
    const LEN: usize = 200_000_000;
    const RUNS: usize = 200;

    print!("{}", INFO);

    println!("# Runtimes of the piped version:");
    let mut counter: usize = 0;
    let piped_durations: Vec<f32> = benchmark_pipe(LEN, RUNS, move || {
        counter += 1;
        piped::Metronome::new(500, 500, 100 * counter, 1_000)
    });

    println!("# Runtimes of the manual version:");
    let mut counter: usize = 0;
    let manual_durations: Vec<f32> = benchmark_pipe(LEN, RUNS, move || {
        counter += 1;
        manual::Metronome::new(500, 500, 100 * counter, 1_000)
    });

    let mean_duration_piped: f32 =
        piped_durations.iter().sum::<f32>() / piped_durations.len() as f32;

    let mean_duration_manual: f32 =
        manual_durations.iter().sum::<f32>() / manual_durations.len() as f32;

    let combined: Vec<(f32, f32)> = Iterator::zip(piped_durations.iter(), manual_durations.iter())
        .map(|(piped, manual)| (piped - manual, piped / manual))
        .collect();

    let (
        mean_difference,
        mean_relation,
        min_difference,
        max_difference,
        min_relation,
        max_relation,
    ) = combined.iter().cloned().fold(
        (
            0.0,
            0.0,
            combined[0].0,
            combined[0].0,
            combined[0].1,
            combined[0].1,
        ),
        |(diff_sum, rel_sum, min_diff, max_diff, min_rel, max_rel), (diff, rel)| {
            (
                diff_sum + diff,
                rel_sum + rel,
                f32::min(min_diff, diff),
                f32::max(max_diff, diff),
                f32::min(min_rel, rel),
                f32::max(max_rel, rel),
            )
        },
    );
    let (mean_difference, mean_relation) = (
        mean_difference / combined.len() as f32,
        mean_relation / combined.len() as f32,
    );

    println!(
        "# Mean duration of a piped execution: {}s",
        mean_duration_piped
    );
    println!(
        "# Mean duration of a manual execution: {}s",
        mean_duration_manual
    );
    println!(
        "# Minimal, Mean, and Maximal difference between a piped and a manual execution: {}s, {}s, {}s",
        min_difference, mean_difference, max_difference
    );
    println!(
        "# Minimal, Mean, and Maximal relation between a piped and a manual exection: {}, {}, {}",
        min_relation, mean_relation, max_relation
    );
}
