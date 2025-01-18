use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lidy__yaml::{LineCol, Yaml, YamlData};

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::file::File;
use crate::lidy::{make_rule_set, Builder, Parser, ParserData};
use crate::result::{Data, LidyResult, Position};
use crate::rule::{apply_predefined_rule, Rule};
use crate::yamlfile::YamlFile;

fn check_merged_node<TV, TB>(
    name: &str,
    last_position: &Position,
    origin_position: &Position,
    subparser: &Parser<TV, TB>,
) -> Option<AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    let rule = subparser.rule_set.get(name);
    if rule.is_none() {
        return Some(Box::new(SimpleError::from_check_result(
            "_merge",
            &format!(
                "unknown rule '{name}' encountered at {} following rules from a _merge keyword",
                LineCol::from(last_position)
            ),
            (origin_position).clone().into(),
        )));
    }
    // check_error, to be returned only if the node is not a map checker
    let check_error = SimpleError::from_check(
        "_merge",
        "reference leads to a non-map-checker node",
        &rule.as_ref()?.node,
    );

    let rule_node = &rule.unwrap().node;
    let last_pos = Position::from_line_col_beginning_only(
        origin_position.filename.clone(),
        rule_node.line_col,
    );

    match &rule_node.data {
        YamlData::String(name) => check_merged_node(name, &last_pos, origin_position, subparser),
        YamlData::Mapping(map) => {
            let is_map_checker = map.iter().any(|(key, _)| {
                if let YamlData::String(key_str) = &key.data {
                    matches!(
                        key_str.as_str(),
                        "_map" | "_mapFacultative" | "_mapOf" | "_merge"
                    )
                } else {
                    false
                }
            });
            if !is_map_checker {
                Some(Box::new(check_error))
            } else {
                None
            }
        }
        _ => Some(Box::new(check_error)),
    }
}

pub fn make_meta_parser_for<TV, TB>(
    subparser: &mut Parser<TV, TB>,
) -> Result<Parser<TV, TB>, AnyBoxedError>
where
    TV: Clone + 'static,
    TB: Builder<TV>,
{
    let meta_schema_file = File::read_local_file("../../lidy.schema.yaml")?;
    let mut meta_schema = YamlFile::new(Rc::new(meta_schema_file));
    meta_schema.unmarshal()?;

    let subparser_ref = Rc::new(RefCell::new(subparser));

    let rule_reference_builder: Builder<TV> = {
        // let subparser_ref = Rc::clone(&subparser_ref);
        let builder_fn = Box::new(
            |input: &LidyResult<TV>| -> Result<Data<TV>, AnyBoxedError> {
                let identifier = match &input.data {
                    Data::String(s) => s.to_string(),
                    _ => return Ok(input.data.clone()),
                };

                if identifier == "expression" {
                    println!(
                        "identifier === 'expression', stack: {:?}",
                        std::backtrace::Backtrace::capture()
                    );
                }

                let mut subparser = subparser_ref.borrow_mut();
                if let Some(rule) = subparser.rule_set.get_mut(&*identifier) {
                    rule.is_used = true;
                } else {
                    let rule_exists = match apply_predefined_rule(
                        &ParserData::<(), ()>::default(),
                        &identifier,
                        &Yaml::default(),
                        true,
                    )?.data {
                        Data::Boolean(exists) => exists,
                        _ => panic!("never, apply_predefined_rule must return a boolean when only_check_if_rule_exists is passed")
                    };
                    if !rule_exists {
                        let rule_listing = subparser
                            .rule_set
                            .keys()
                            .map(|k| k.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(Box::new(SimpleError::from_check_result(
                            &identifier,
                            &format!(
                                "encountered unknown rule identifier '{}'. Known rules are: [{}]",
                                identifier, rule_listing
                            ),
                            LineCol {
                                line: input.position.line,
                                column: input.position.column,
                            },
                        )));
                    }
                }
                Ok(input.data.clone())
            },
        )
            as Box<dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<(dyn std::error::Error)>>>;
        Rc::new(RefCell::new(builder_fn))
    };

    let map_checker_builder: Builder<TV> = {
        let subparser_ref = Rc::clone(&subparser_ref);
        let builder_fn = Box::new(
            |input: &LidyResult<TV>| -> Result<Data<TV>, AnyBoxedError> {
                if let Data::MapData(map_data) = &input.data {
                    if let Some(merge) = map_data.map.get("_merge") {
                        let mut join_error = JoinError::default();
                        if let Data::ListData(list_data) = &merge.data {
                            let subparser = subparser_ref.borrow();
                            for result in &list_data.list_of {
                                match &result.data {
                                    Data::CustomData(_) => continue,
                                    Data::String(s) => {
                                        if let Some(err) = check_merged_node(
                                            s,
                                            &result.position,
                                            &result.position,
                                            &subparser,
                                        ) {
                                            join_error.add(err);
                                        }
                                    }
                                    _ => continue,
                                }
                            }
                        }
                        join_error.into_result()?;
                    }
                }
                Ok(input.data.clone())
            },
        )
            as Box<dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<(dyn std::error::Error)>>>;
        Rc::new(RefCell::new(builder_fn))
    };

    let sized_checker_keyword_set_builder: Builder<TV> = {
        let builder_fn = Box::new(
            |input: &LidyResult<TV>| -> Result<Data<TV>, AnyBoxedError> {
                if let Data::MapData(map_data) = &input.data {
                    for keyword in &["_min", "_max", "_nb"] {
                        if let Some(value) = map_data.map.get(*keyword) {
                            if let Data::Float(n) = &value.data {
                                if *n < 0.0 {
                                    return Err(Box::new(SimpleError::from_check_result(
                                        keyword,
                                        "cannot be negative",
                                        LineCol {
                                            line: input.position.line,
                                            column: input.position.column,
                                        },
                                    )));
                                }
                            }
                        }
                    }
                    let min = map_data.map.get("_min").and_then(|v| match v.data {
                        Data::Float(n) => Some(n),
                        _ => None,
                    });
                    let max = map_data.map.get("_max").and_then(|v| match v.data {
                        Data::Float(n) => Some(n),
                        _ => None,
                    });
                    let nb = map_data.map.get("_nb").and_then(|v| match v.data {
                        Data::Float(n) => Some(n),
                        _ => None,
                    });

                    if nb.is_some() {
                        let mut min_or_max = "";
                        if min.is_some() {
                            min_or_max = "min";
                        } else if max.is_some() {
                            min_or_max = "max";
                        }

                        return Err(Box::new(SimpleError::from_check_result(
                            "_nb",
                            &format!(
                                "it makes no sense to use the `_nb` and `_{min_or_max}` together"
                            ),
                            LineCol {
                                line: input.position.line,
                                column: input.position.column,
                            },
                        )));
                    }

                    if let (Some(min), Some(max)) = (min, max) {
                        if min > max {
                            return Err(Box::new(SimpleError::from_check_result(
                                "_min",
                                "`_max` cannot be lower than `_min`",
                                LineCol {
                                    line: input.position.line,
                                    column: input.position.column,
                                },
                            )));
                        }
                    }
                }
                Ok(input.data.clone())
            },
        )
            as Box<dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<(dyn std::error::Error)>>>;
        Rc::new(RefCell::new(builder_fn))
    };

    let meta_builder_map: HashMap<Box<str>, Builder<TV>> = HashMap::from([
        (Box::from("ruleReference"), rule_reference_builder),
        (Box::from("mapChecker"), map_checker_builder),
        (
            Box::from("sizedCheckerKeywordSet"),
            sized_checker_keyword_set_builder,
        ),
    ]);

    let rule_set = make_rule_set(&meta_schema, meta_builder_map)?;
    Ok(Parser { rule_set })
}

pub fn check_rule_set(rule_set: &mut HashMap<Box<str>, Rule>) -> Result<(), AnyBoxedError> {
    let mut join_error = JoinError::default();

    // Check main rule exists and mark it as used
    if let Some(main_rule) = rule_set.get_mut("main") {
        main_rule.is_used = true;
    } else {
        join_error.add(Box::new(SimpleError::from_str(
            "could not find the 'main' rule",
        )));
    }

    // Check for unused rules
    for (name, rule) in rule_set.iter() {
        if !rule.is_used {
            join_error.add(Box::new(SimpleError::from_str(&format!(
                "rule '{}' is defined but never used",
                name
            ))));
        }
    }

    // Check for direct rule references
    for (name, rule) in rule_set.iter() {
        if let Some(err) = check_direct_rule_reference(rule_set, &rule.node, &[&**name]) {
            join_error.add(err);
        }
    }

    join_error.into_result()
}

fn check_direct_rule_reference(
    rule_set: &HashMap<Box<str>, Rule>,
    rule_node: &Yaml,
    rule_name_array: &[&str],
) -> Option<AnyBoxedError> {
    match &rule_node.data {
        YamlData::String(s) => {
            // Check for self-reference
            if rule_name_array.contains(&s.as_str()) {
                return Some(Box::new(SimpleError::from_str(&format!(
                    "rule '{}' references itself",
                    s
                ))));
            }

            // Check target rule
            if let Some(target_rule) = rule_set.get(&**s) {
                check_direct_rule_reference(
                    rule_set,
                    &target_rule.node,
                    &rule_name_array
                        .iter()
                        .chain(std::iter::once(&s.as_str()))
                        .copied()
                        .collect::<Vec<_>>(),
                )
            } else {
                None // Predefined rule
            }
        }
        YamlData::Mapping(map) => {
            let mut join_error = JoinError::default();

            // Check for _oneOf or _merge nodes
            for (key, value) in map {
                if let YamlData::String(key_str) = &key.data {
                    if key_str == "_oneOf" || key_str == "_merge" {
                        if let YamlData::List(seq) = &value.data {
                            for node in seq {
                                if let Some(err) =
                                    check_direct_rule_reference(rule_set, node, rule_name_array)
                                {
                                    join_error.add(err);
                                }
                            }
                        }
                    }
                }
            }

            match join_error.into_result() {
                Ok(_) => None,
                Err(e) => Some(e),
            }
        }
        _ => Some(Box::new(SimpleError::from_str(
            "rule node should be either a scalar or a mapping",
        ))),
    }
}
