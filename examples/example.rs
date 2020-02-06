use pipes::slice::*;
use pipes::util::*;
use pipes::{Pipe, Pipeline};

const LENGTH: usize = 128;

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

    // Checking the input and output of the pipe by contraining it.
    type InputPipe<P> = PipeConstraint<(), Option<(f32, f32)>, P>;
    let input: InputPipe<_> = input.constraint();

    // Creating the output part of the pipe.
    let output = (
        SliceConsumer::new(output0.as_mut()),
        SliceConsumer::new(output1.as_mut()),
    )
        // Making consumption optional.
        .optional()
        // Evaluating the consumption result, useful for the processing loop.
        .post_map(|output: Option<(ConsumeResult, ConsumeResult)>| {
            if let Some((left_result, right_result)) = output {
                left_result == ConsumeResult::Ok && right_result == ConsumeResult::Ok
            } else {
                false
            }
        });

    // Checking the input and output of the pipe by contraining it.
    type OutputPipe<P> = PipeConstraint<Option<(f32, f32)>, bool, P>;
    let output: OutputPipe<_> = output.constraint();

    // Lazily creating a rendering pipeline.
    type ProcessPipe<P> = PipeConstraint<Option<(f32, f32)>, Option<(f32, f32)>, P>;
    let process: ProcessPipe<_> =
        Lazy::new(|(left, right): (f32, f32)| (left + right, -1.0 * (left + right)))
            .optional()
            .constraint();

    // Completing the pipeline and running it.
    process.pre_connect(input).post_connect(output).run();

    // Validating the result.
    for ((i0, i1), (o0, o1)) in Iterator::zip(
        Iterator::zip(input0.iter(), input1.iter()),
        Iterator::zip(output0.iter(), output1.iter()),
    ) {
        assert_eq!(*i0 + *i1, *o0);
        assert_eq!((-1.0) * (*i0 + *i1), *o1);
    }
}
