use crate::*;

#[test]
fn trait_object() {
    let mut pipe: Box<dyn Pipe<InputItem = (), OutputItem = Option<usize>>> =
        Box::new(PipeIter::new((0..42).map(|_| 42)));

    while let Some(i) = pipe.next(()) {
        assert_eq!(i, 42);
    }
}
