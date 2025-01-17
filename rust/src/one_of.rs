use crate::{
    error::{AnyBoxedError, JoinError, SimpleError},
    expression::apply_expression,
    lidy::ParserData,
    LidyResult,
};
use lidy__yaml::{Yaml, YamlData};

pub fn apply_one_of_matcher<T>(
    parser_data: &mut ParserData<T>,
    node: &Yaml,
    content: &Yaml,
) -> Result<LidyResult<T>, AnyBoxedError>
where
    T: Clone,
{
    let items = match &node.data {
        YamlData::List(list) => list,
        _ => panic!("expected schema node to be a sequence for (_oneOf: <x>)"),
    };

    let mut join_error = JoinError::default();
    join_error.add(
        SimpleError::from_check(
            "_oneOf",
            &format!("none of the {} expressions matched", items.len()),
            content,
        )
        .into(),
    );

    for schema in items {
        match apply_expression(parser_data, schema, content) {
            Ok(result) => return Ok(result),
            Err(error) => join_error.add(error),
        }
    }

    Err(join_error.into())
}
