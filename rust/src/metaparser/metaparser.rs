use std::collections::HashMap;
use std::rc::Rc;

use lidy__yaml::{Yaml, YamlData};

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::file::File;
use crate::parser::{make_rule_set, Parser};
use crate::result::Data;
use crate::rule::Rule;
use crate::yamlfile::YamlFile;
use crate::LidyResult;

pub fn make_meta_parser_for<'a, 'b, TV>(
    parser: &'b mut Parser<'a, TV>,
) -> Result<Parser<'b, ()>, AnyBoxedError>
where
    'a: 'b,
{
    let meta_schema_file = File::read_local_file("../lidy.schema.yaml")?;
    let mut meta_schema = YamlFile::new(Rc::new(meta_schema_file));
    meta_schema.deserialize()?;

    let meta_rule_set = make_rule_set(&meta_schema)?;

    let meta_parser = Parser {
        content_file_name: "lidy.schema.yaml".into(),
        rule_set: meta_rule_set,
        rule_trace: Vec::new(),
        rule_is_matching_node: HashMap::new(),
        builder_callback: Box::new(
            |rule_name,
             lidy_result: &LidyResult<()>|
             -> Result<Data<()>, Box<dyn std::error::Error>> {
                match rule_name {
                    "mapChecker" => parser.run_map_checker_builder(lidy_result),
                    "ruleReference" => parser.run_rule_reference_checker_builder(lidy_result),
                    "sizeCheckerKeywordSet" => parser.run_size_checker_builder(lidy_result),
                    _ => Ok(lidy_result.data.clone()),
                }
            },
        ),
    };

    Ok(meta_parser)
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
