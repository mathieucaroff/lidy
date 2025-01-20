use lidy__yaml::{Yaml, YamlData};
use regex::Regex;

use crate::result::Data;
use crate::{error::AnyBoxedError, parser::Parser, LidyResult, SimpleError};

pub fn apply_regex_matcher<TV>(
    parser: &mut Parser<TV>,
    node: &Yaml,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone + 'static,
{
    // Obtain the regex pattern as string from the schema node
    let pattern = match &node.data {
        YamlData::String(s) => s,
        _ => panic!("expected schema node to be a string for (_regex: <pattern>)"),
    };
    // Compile the regex pattern
    let regex = Regex::new(pattern).expect("invalid regex pattern");

    // Check that the content node is a string
    let content_str = match &content.data {
        YamlData::String(s) => s,
        _ => {
            return Err(SimpleError::from_check("_regex", "must be a string", content).into());
        }
    };

    // Test the regex against the content
    if regex.is_match(content_str) {
        Ok(LidyResult::create(
            parser,
            content,
            Data::String(content_str.clone().into()),
        ))
    } else {
        Err(SimpleError::from_check(
            "_regex",
            &format!("must match regex /{}/", pattern),
            content,
        )
        .into())
    }
}
