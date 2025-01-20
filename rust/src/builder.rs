use crate::{
    error::AnyBoxedError,
    result::{Data, LidyResult},
};

pub trait BuilderTrait<TV>: Clone
where
    TV: Clone + 'static,
{
    fn build(&mut self, lidy_result: &LidyResult<TV>) -> Result<Data<TV>, AnyBoxedError>;
}

pub struct Builder<TV>(pub Box<dyn BuilderTrait<TV>>)
where
    TV: Clone;
