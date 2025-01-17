use crate::any::map_any_yaml_data_to_lidy_data;
use crate::error::AnyBoxedError;
use crate::expression::apply_expression;
use crate::lidy::{Builder, ParserData, RuleNodePair};
use crate::result::{Data, LidyResult, Position};
use lidy__yaml::{Yaml, YamlData};
use regex::Regex;

lazy_static::lazy_static! {
    static ref REGEX_BASE64: Regex = Regex::new(r"^[a-zA-Z0-9_\- \n]*[= \n]*$").unwrap();
}

#[derive(Clone)]
pub struct Rule<T>
where
    T: Clone,
{
    // Name of the rule in the schema
    pub name: Box<str>,
    // Node associated to the rule in the schema
    pub node: Yaml,
    // Builder given by the user for that rule, if any
    pub builder: Option<Builder<T>>,
    // Whether the rule is referenced anywhere in the schema. This is used
    // in the metaparser to report unused rules
    pub is_used: bool,
}

pub fn apply_rule<T>(
    parser_data: &mut ParserData<T>,
    rule_name: &str,
    content: &Yaml,
) -> Result<LidyResult<T>, AnyBoxedError>
where
    T: Clone + 'static,
{
    parser_data.rule_trace.push(rule_name.into());

    let rule_node_pair = RuleNodePair::new(rule_name.into(), content);

    let result = match parser_data.parser.rule_set.get(rule_name) {
        None => apply_predefined_rule(parser_data, rule_name, content, false),
        Some(rule) => {
            let rule = rule.clone();

            // Detect infinite loops while processing the data
            let has_loop = parser_data
                .rule_is_matching_node
                .contains_key(&rule_node_pair);

            if has_loop {
                return Err(format!(
                    "Infinite loop: Rule {} encountered multiple times for the same node ({:?})",
                    rule_name, content
                )
                .into());
            }

            parser_data
                .rule_is_matching_node
                .insert(rule_node_pair.clone(), ());
            let mut lidy_result = apply_expression(parser_data, &rule.node, content)?;

            parser_data.rule_is_matching_node.remove(&rule_node_pair);

            if let Some(builder) = rule.builder {
                lidy_result.data = builder.borrow_mut()(&lidy_result)?;
            }

            Ok(lidy_result)
        }
    };
    parser_data.rule_trace.pop();
    result
}

type RuleResult<T> = Result<Data<T>, AnyBoxedError>;
type PredefinedRuleFn<'a, T> = Box<dyn 'a + FnOnce(&Yaml) -> RuleResult<T>>;

pub fn apply_predefined_rule<T>(
    parser_data: &ParserData<T>,
    rule_name: &str,
    content: &Yaml,
    only_check_if_rule_exists: bool,
) -> Result<LidyResult<T>, AnyBoxedError>
where
    T: Clone,
{
    let predefined_rule: Option<PredefinedRuleFn<T>> = match rule_name {
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
                match value.parse() {
                    Ok(value) => Ok(Data::Float(value)),
                    Err(e) => {
                        panic!("failed to parse Yaml Real into Rust f64 float: {e}")
                    }
                }
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
        "anyData" => Some(Box::new(|content: &Yaml| {
            Ok(map_any_yaml_data_to_lidy_data(
                &parser_data.content_file_name,
                &parser_data.rule_trace.last().unwrap(),
                content,
            ))
        })),
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
            Ok(data) => Ok(LidyResult::create(&parser_data, content, data)),
            Err(e) => Err(e),
        },
    }
}
