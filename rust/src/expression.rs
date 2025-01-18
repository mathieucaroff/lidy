use lidy__yaml::{Yaml, YamlData};

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::in_::apply_in_matcher;
use crate::lidy::{Builder, ParserData};
use crate::list::apply_list_matcher;
use crate::map::apply_map_matcher;
use crate::one_of::apply_one_of_matcher;
use crate::range::apply_range_matcher;
use crate::regex::apply_regex_matcher;
use crate::result::LidyResult;
use crate::rule::apply_rule;
use crate::size::apply_size_check;

pub fn apply_expression<TV, TB>(
    parser_data: &mut ParserData<TV, TB>,
    schema: &Yaml,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    match &schema.data {
        YamlData::String(value) => apply_rule(parser_data, value, content),
        YamlData::Mapping(mapping) => {
            let mut map = None;
            let mut map_facultative = None;
            let mut map_of = None;
            let mut merge = None;
            let mut list = None;
            let mut list_facultative = None;
            let mut list_of = None;
            let mut min = None;
            let mut max = None;
            let mut nb = None;

            for (key, value) in mapping {
                if let YamlData::String(key_str) = &key.data {
                    match key_str.as_str() {
                        "_regex" => return apply_regex_matcher(parser_data, value, content),
                        "_in" => return apply_in_matcher(parser_data, value, content),
                        "_range" => return apply_range_matcher(parser_data, value, content),
                        "_oneOf" => return apply_one_of_matcher(parser_data, value, content),
                        "_map" => map = Some(value),
                        "_mapFacultative" => map_facultative = Some(value),
                        "_mapOf" => map_of = Some(value),
                        "_merge" => merge = Some(value),
                        "_list" => list = Some(value),
                        "_listFacultative" => list_facultative = Some(value),
                        "_listOf" => list_of = Some(value),
                        "_min" => min = Some(value),
                        "_max" => max = Some(value),
                        "_nb" => nb = Some(value),
                        _ => {
                            return Err(SimpleError::from_message(
                                format!("Unknown keyword found in matcher: '{key_str}'").into(),
                            )
                            .into())
                        }
                    }
                }
            }

            let mut join_error = JoinError::default();
            if min.is_some() || max.is_some() || nb.is_some() {
                if let Some(error) = apply_size_check(content, min, max, nb) {
                    join_error.add(error);
                }
            }

            let mut result = None;

            if map.is_some() || map_facultative.is_some() || map_of.is_some() || merge.is_some() {
                result = Some(apply_map_matcher(
                    parser_data,
                    map,
                    map_facultative,
                    map_of,
                    merge,
                    content,
                )?);
            }

            if list.is_some() || list_facultative.is_some() || list_of.is_some() {
                result = Some(apply_list_matcher(
                    parser_data,
                    list,
                    list_facultative,
                    list_of,
                    content,
                )?);
            }

            result.ok_or_else(|| {
                SimpleError::from_message("No keyword found in matcher".into()).into()
            })
        }
        _ => panic!("Lidy expressions must be strings (rule names) or mappings (checkers)"),
    }
}
