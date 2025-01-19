use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lidy__yaml::{LineCol, Yaml, YamlData};

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::file::File;
use crate::lidy::{make_rule_set, Parser};
use crate::result::{Data, LidyResult};
use crate::rule::{apply_predefined_rule, Rule};
use crate::yamlfile::YamlFile;

struct RuleReferenceBuilder<'a, TV>
where
    TV: Clone,
{
    subparser: &'a mut Parser<TV>,
}

pub fn make_meta_parser_for<TV>(subparser: &mut Parser<TV>) -> Result<Parser<()>, AnyBoxedError>
where
    TV: Clone,
{
    let meta_schema_file = File::read_local_file("../../lidy.schema.yaml")?;
    let mut meta_schema = YamlFile::new(Rc::new(meta_schema_file));
    meta_schema.deserialize()?;

    let rule_reference_builder: Builder<()> = {
        let builder_fn: Box<dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<dyn Error>>> =
            Box::new(
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

                    if let Some(rule) = subparser.rule_set.get_mut(&*identifier) {
                        rule.is_used = true;
                    } else {
                        let rule_exists = match apply_predefined_rule(
                        &Parser::<(), ()>::default(),
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
                as Box<
                    dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<(dyn std::error::Error)>>,
                >;
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

    let mut meta_parser = Parser {
        content_file_name: "lidy.schema.yaml".into(),
        rule_set,
        builder_map: HashMap::new(),
        rule_trace: Vec::new(),
        rule_is_matching_node: HashMap::new(),
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
