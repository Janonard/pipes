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

    // Creating the input part of the pipeline.
    type InputPipe<P> = PipeConstraint<(), Option<(f32, f32)>, P>;
    let input: InputPipe<_> = {
        let left_producer = SliceProducer::new(input0.as_ref());
        let right_producer = SliceProducer::new(input1.as_ref());

        ().constraint()
            >> Lazy::new(|_: ()| ((), ()))
            >> (left_producer, right_producer)
            >> Lazy::new(|(left, right): (Option<&f32>, Option<&f32>)| Some((*(left?), *(right?))))
    };

    // Creating the output part of the pipeline.
    type OutputPipe<P> = PipeConstraint<Option<(f32, f32)>, bool, P>;
    let output: OutputPipe<_> = {
        let left_consumer = SliceConsumer::new(output0.as_mut());
        let right_consumer = SliceConsumer::new(output1.as_mut());

        fn evaluate_result(result: Option<(ConsumeResult, ConsumeResult)>) -> bool {
            if let Some((left_result, right_result)) = result {
                left_result == ConsumeResult::Ok && right_result == ConsumeResult::Ok
            } else {
                false
            }
        }

        (left_consumer, right_consumer).optional().constraint() >> Lazy::new(evaluate_result)
    };

    // Lazily creating a rendering pipeline.
    type ProcessPipe<P> = PipeConstraint<Option<(f32, f32)>, Option<(f32, f32)>, P>;
    let process: ProcessPipe<_> =
        Lazy::new(|(left, right): (f32, f32)| (left + right, -1.0 * (left + right)))
            .optional()
            .constraint();

    // Completing the pipeline and running it.
    (input >> process >> output).run();

    // Validating the result.
    for ((i0, i1), (o0, o1)) in Iterator::zip(
        Iterator::zip(input0.iter(), input1.iter()),
        Iterator::zip(output0.iter(), output1.iter()),
    ) {
        assert_eq!(*i0 + *i1, *o0);
        assert_eq!((-1.0) * (*i0 + *i1), *o1);
    }
}
