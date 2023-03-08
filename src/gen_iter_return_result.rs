use std::{ops::Generator, rc::Rc};

use gen_iter::GenIterReturn;

use crate::WaveCollapseError;

pub trait GenIterReturnResult<T> {
    fn calc_result(self) -> Result<Rc<T>, WaveCollapseError>;
}

trait ResultRc<T> {
    fn result(self) -> Result<Rc<T>, WaveCollapseError>;
}

impl<T> ResultRc<T> for Result<Rc<T>, WaveCollapseError> {
    fn result(self) -> Result<Rc<T>, WaveCollapseError> {
        self
    }
}

impl<T, G: Generator + Unpin> GenIterReturnResult<T> for GenIterReturn<G>
where
    G::Return: ResultRc<T>,
{
    fn calc_result(self) -> Result<Rc<T>, WaveCollapseError> {
        // FIXME: for some reason I'm not allowed to use self in this function.
        //      Moving it into foo seems to fix the issue
        let mut foo = self;
        while let Some(_s) = Iterator::next(&mut &mut foo) {}

        match foo.return_or_self() {
            Ok(r) => match r.result() {
                Ok(r) => Ok(r),
                Err(e) => Err(e),
            },
            Err(_) => Err(WaveCollapseError::IterationError),
        }
    }
}
