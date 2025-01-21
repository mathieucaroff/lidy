use crate::{
    error::AnyBoxedError,
    result::{Data, LidyResult},
};
use std::collections::HashMap;

pub trait BuilderTrait<TV> {
    fn build(&mut self, lidy_result: &LidyResult<TV>) -> Result<Data<TV>, AnyBoxedError>;
}

pub type BuilderMap<TV> = HashMap<Box<str>, Box<dyn BuilderTrait<TV>>>;
