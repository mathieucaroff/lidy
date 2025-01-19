use lazy_static::lazy_static;
use lidy__yaml::{Yaml, YamlData};
use regex::Regex;

use crate::result::Data;
use crate::{
    error::AnyBoxedError,
    lidy::{Builder, Parser},
    LidyResult, SimpleError,
};

lazy_static! {
    static ref RANGE_REGEX: Regex =
        Regex::new(r"(([0-9]+(\.[0-9]+)?) *(<=?) *)?(int|float)( *(<=?) *([0-9]+(\.[0-9]+)?))?",)
            .unwrap();
}

pub fn apply_range_matcher<TV>(
    parser: &mut Parser<TV>,
    node: &Yaml,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone,
{
    // Check that content is a number
    let value = match &content.data {
        YamlData::Integer(i) => *i as f64,
        YamlData::Real(r) => r.parse::<f64>().unwrap(),
        _ => {
            return Err(SimpleError::from_check("_range", "must be a number", content).into());
        }
    };

    let pattern_error =
        "the range pattern must be a valid range matcher string (_range: <pattern>)";

    // Get the pattern from the schema node
    let pattern = match &node.data {
        YamlData::String(s) => s,
        _ => panic!("{}", pattern_error),
    };

    // Parse the range pattern
    let captures = RANGE_REGEX.captures(pattern).expect(pattern_error);

    let left_boundary: Option<f64> = captures.get(2).map(|m| m.as_str().parse::<f64>().unwrap());
    let left_operator = captures.get(4).map_or("", |m| m.as_str());
    let number_type = captures.get(5).unwrap().as_str();
    let right_operator = captures.get(7).map_or("", |m| m.as_str());
    let right_boundary: Option<f64> = captures.get(8).map(|m| m.as_str().parse::<f64>().unwrap());

    // Validate number type
    if number_type == "int" && !value.trunc().eq(&value) {
        return Err(SimpleError::from_check("_range", "must be an integer", content).into());
    }

    // Check boundaries
    let mut ok = true;
    if let Some(left) = left_boundary {
        ok = ok
            && match left_operator {
                "<" => left < value,
                "<=" => left <= value,
                _ => true,
            };
    }

    if let Some(right) = right_boundary {
        ok = ok
            && match right_operator {
                "<" => value < right,
                "<=" => value <= right,
                _ => true,
            };
    }

    if !ok {
        return Err(SimpleError::from_check(
            "_range",
            "must be inside the specified range",
            content,
        )
        .into());
    }

    let data = match &content.data {
        YamlData::Integer(int) => Data::Integer(*int),
        YamlData::Real(_) => Data::Float(value),
        _ => panic!("never, content is no longer a number"),
    };

    Ok(LidyResult::create(parser, content, data))
}
