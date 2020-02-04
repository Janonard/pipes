use pipes::slice::*;
use pipes::util::*;
use pipes::{Pipe, Pipeline};

const LENGTH: usize = 128;

struct Input<P: Pipe<InputItem = (), OutputItem = Option<(f32, f32)>>> {
    pipe: P,
}

impl<P> Pipe for Input<P>
where
    P: Pipe<InputItem = (), OutputItem = Option<(f32, f32)>>,
{
    type InputItem = ();
    type OutputItem = Option<(f32, f32)>;

    fn next(&mut self, _: ()) -> Option<(f32, f32)> {
        self.pipe.next(())
    }
}

struct Output<P: Pipe<InputItem = Option<(f32, f32)>, OutputItem = bool>> {
    pipe: P,
}

impl<P> Pipe for Output<P>
where
    P: Pipe<InputItem = Option<(f32, f32)>, OutputItem = bool>,
{
    type InputItem = Option<(f32, f32)>;
    type OutputItem = bool;

    fn next(&mut self, item: Option<(f32, f32)>) -> bool {
        self.pipe.next(item)
    }
}

fn main() {
    // Allocating input and output data.
    let input0: Vec<f32> = (0..LENGTH).map(|i| i as f32).collect();
    let input1: Vec<f32> = (0..LENGTH).map(|i| (LENGTH - i) as f32).collect();
    let mut output0: Vec<f32> = vec![0.0; LENGTH];
    let mut output1: Vec<f32> = vec![0.0; LENGTH];

    // Creating the input part of the pipe.
    let input = (
        SliceProducer::new(input0.as_ref()),
        SliceProducer::new(input1.as_ref()),
    )
        // Widening the void to two channels.
        .pre_map(|_: ()| ((), ()))
        // Derefing the references, wrapping the values in a general option.
        .post_map(|(left, right)| Some((*(left?), *(right?))));
    let input = Input { pipe: input };

    // Creating the output part of the pipe.
    let output = (
        SliceConsumer::new(output0.as_mut()),
        SliceConsumer::new(output1.as_mut()),
    )
        // Making consumption optional.
        .optional()
        // Evaluating the consumption result, usefull for the processing loop.
        .post_map(|output: Option<(ConsumeResult, ConsumeResult)>| {
            if let Some((left_result, right_result)) = output {
                left_result == ConsumeResult::Ok && right_result == ConsumeResult::Ok
            } else {
                false
            }
        });

    // Lazily creating a rendering pipeline.
    let process = Lazy::new(|(left, right): (f32, f32)| (left + right, -1.0 * (left + right))).optional();

    process.pre_connect(input).post_connect(output).run();
}
