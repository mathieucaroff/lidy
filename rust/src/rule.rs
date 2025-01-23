use crate::any::map_any_yaml_data_to_lidy_data;
use crate::error::AnyBoxedError;
use crate::expression::apply_expression;
use crate::parser::{Parser, RuleNodePair};
use crate::result::{Data, LidyResult, Position};
use crate::syaml::must_parse_float;
use crate::SimpleError;
use lidy__yaml::{Yaml, YamlData};
use regex::Regex;

lazy_static::lazy_static! {
    static ref REGEX_BASE64: Regex = Regex::new(r"^[a-zA-Z0-9_\- \n]*[= \n]*$").unwrap();
}

#[derive(Clone, Debug)]
pub struct Rule {
    // Name of the rule in the schema
    pub name: Box<str>,
    // Node associated to the rule in the schema
    pub node: Yaml,
    // Whether the rule is referenced anywhere in the schema. This is used
    // in the metaparser to report unused rules
    pub is_used: bool,
}

pub fn apply_rule<TV>(
    parser: &mut Parser<TV>,
    rule_name: &str,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
{
    parser.rule_trace.push(rule_name.into());

    let result = try_apply_rule(parser, rule_name, content);
    parser.rule_trace.pop();
    result
}

fn try_apply_rule<TV>(
    parser: &mut Parser<'_, TV>,
    rule_name: &str,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError> {
    let rule_node_pair = RuleNodePair::new(rule_name.into(), content);
    match parser.rule_set.get(rule_name) {
        None => apply_predefined_rule(parser, rule_name, content, false),
        Some(rule) => {
            let rule = rule.clone();

            // Detect infinite loops while processing the data
            let has_loop = parser.rule_is_matching_node.contains_key(&rule_node_pair);

            if has_loop {
                return Err(format!(
                    "Infinite loop: Rule {} encountered multiple times for the same node ({:?})",
                    rule_name, content
                )
                .into());
            }

            parser
                .rule_is_matching_node
                .insert(rule_node_pair.clone(), ());
            let mut lidy_result = apply_expression(parser, &rule.node, content).map_err(|err| {
                SimpleError::from_check_result(rule_name, &err.to_string(), content.line_col)
            })?;

            parser.rule_is_matching_node.remove(&rule_node_pair);

            lidy_result.data = (parser.builder_callback)(rule_name, &lidy_result)?;

            Ok(lidy_result)
        }
    }
}

type RuleResult<TV> = Result<Data<TV>, AnyBoxedError>;
type PredefinedRuleFn<'a, TV> = Box<dyn 'a + FnOnce(&Yaml) -> RuleResult<TV>>;

pub fn apply_predefined_rule<TV>(
    parser: &mut Parser<TV>,
    rule_name: &str,
    content: &Yaml,
    only_check_if_rule_exists: bool,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
{
    let predefined_rule: Option<PredefinedRuleFn<TV>> = match rule_name {
        "string" => Some(Box::new(|content: &Yaml| {
            if let YamlData::String(value) = &content.data {
                Ok(Data::String(value.clone().into()))
            } else {
                Err(("expected a string").into())
            }
        })),
        "int" => Some(Box::new(|content: &Yaml| {
            if let YamlData::Integer(value) = &content.data {
                Ok(Data::Integer(*value))
            } else {
                Err("expected an integer".into())
            }
        })),
        "float" => Some(Box::new(|content: &Yaml| {
            if let YamlData::Real(value) = &content.data {
                Ok(Data::Float(must_parse_float(value)))
            } else if let YamlData::Integer(value) = &content.data {
                Ok(Data::Float(*value as f64))
            } else {
                Err("expected a float".into())
            }
        })),
        "binary" => Some(Box::new(|content: &Yaml| {
            if let YamlData::String(value) = &content.data {
                if REGEX_BASE64.is_match(value) {
                    Ok(Data::String((value.clone()).into()))
                } else {
                    Err("expected a base64 value".into())
                }
            } else {
                Err("expected a binary or string value".into())
            }
        })),
        "boolean" => Some(Box::new(|content: &Yaml| {
            if let YamlData::Boolean(b) = &content.data {
                Ok(Data::Boolean(*b))
            } else {
                Err("expected a boolean".into())
            }
        })),
        "nullType" => Some(Box::new(|content: &Yaml| {
            if let YamlData::Null = content.data {
                Ok(Data::Null)
            } else {
                Err("expected the null value".into())
            }
        })),
        "timestamp" => Some(Box::new(|content: &Yaml| {
            if let YamlData::String(value) = &content.data {
                match chrono::DateTime::parse_from_rfc3339(value) {
                    Ok(_) => Ok(Data::String((value.clone()).into())),
                    Err(_) => Err("invalid timestamp format - must be RFC3339/ISO8601".into()),
                }
            } else {
                Err("expected a timestamp string (an ISO 8601 datetime)".into())
            }
        })),
        "any" => Some(Box::new(|_: &Yaml| Ok(Data::Null))),
        "anyData" => {
            let filename = parser.content_file_name.clone();
            let rule_name = parser.rule_trace.last().unwrap();

            Some(Box::new(move |content: &Yaml| {
                Ok(map_any_yaml_data_to_lidy_data(
                    &filename, rule_name, content,
                ))
            }))
        }
        "never" => Some(Box::new(|_: &Yaml| {
            Err("encountered the never value".into())
        })),
        _ => None,
    };

    if only_check_if_rule_exists {
        return Ok(LidyResult {
            rule_name: "".into(),
            position: Position::default(),
            data: Data::Boolean(predefined_rule.is_some()),
        });
    }

    match predefined_rule {
        None => Err(format!("rule '{rule_name}' not found in the schema").into()),
        Some(rule_fn) => match rule_fn(content) {
            Ok(data) => Ok(LidyResult::create(&parser, content, data)),
            Err(e) => Err(e),
        },
    }
}
