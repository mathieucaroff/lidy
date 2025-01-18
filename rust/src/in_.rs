use crate::{
    any::map_any_yaml_data_to_lidy_data,
    error::AnyBoxedError,
    lidy::{Builder, ParserData},
    syaml, LidyResult, SimpleError,
};
use lidy__yaml::{Yaml, YamlData};

pub fn apply_in_matcher<TV, TB>(
    parser_data: &mut ParserData<TV, TB>,
    node: &Yaml,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    let valid_value_list = match &node.data {
        YamlData::List(list) => list,
        _ => panic!("expected schema node to be a sequence for (_in: <x>)"),
    };

    if !syaml::is_scalar(content) {
        let error = SimpleError::from_check("_in", "must be a scalar node", content);
        return Err(error.into());
    }

    for valid_value in valid_value_list {
        if valid_value.data == content.data {
            return Ok(LidyResult::create(
                parser_data,
                content,
                map_any_yaml_data_to_lidy_data(
                    &parser_data.content_file_name,
                    parser_data.rule_trace.last().unwrap(),
                    content,
                ),
            ));
        };
    }

    return Err(SimpleError::from_check(
        "_in",
        &format!(
            "must be one of the accepted values ({:?}) but is {:?}",
            valid_value_list, content.data
        ),
        content,
    )
    .into());
}
