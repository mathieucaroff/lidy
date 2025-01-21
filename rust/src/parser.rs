use lidy__yaml::{LineCol, Yaml, YamlData};
use std::collections::HashMap;
use std::rc::Rc;

use crate::builder::BuilderMap;
use crate::error::{AnyBoxedError, SimpleError};
use crate::file::File;
use crate::metaparser::{check_rule_set, make_meta_parser_for};
use crate::rule::Rule;
use crate::yamlfile::YamlFile;

#[derive(Clone, Default)]
pub struct Parser<TV>
where
    TV: Clone + 'static,
{
    pub content_file_name: Rc<str>,
    // The map of rule name to rule content
    pub rule_set: HashMap<Box<str>, Rule>,
    // The map of builder functions for each rule
    pub builder_map: BuilderMap<TV>,
    // The stack of the names of the rules
    pub rule_trace: Vec<Box<str>>,
    // Whether this rule is already being processed for a node. This is used
    // to detect infinite loops
    pub rule_is_matching_node: HashMap<RuleNodePair, ()>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct RuleNodePair {
    pub rule: Box<str>,
    pub node_line_col: LineCol,
}

impl RuleNodePair {
    pub fn new(rule: Box<str>, node: &Yaml) -> Self {
        Self {
            rule,
            node_line_col: node.line_col,
        }
    }
}

impl<TV> Parser<TV>
where
    TV: Clone + 'static,
{
    pub fn make(file: &Rc<File>, builder_map: BuilderMap<TV>) -> Result<Self, AnyBoxedError> {
        let mut schema_file = YamlFile::new(file.clone());
        schema_file.deserialize()?;

        let rule_set = make_rule_set(&schema_file)?;
        let mut parser = Parser {
            content_file_name: file.name.into(),
            rule_set,
            builder_map,
            rule_trace: Vec::new(),
            rule_is_matching_node: HashMap::new(),
        };

        // METAPARSING VALIDATION
        // Validate that the provided schema is valid according to the lidy metaschema
        let meta_parser = make_meta_parser_for::<TV>(&mut parser)?;
        meta_parser._parse_data(&schema_file)?;
        check_rule_set(&mut parser.rule_set);

        Ok(parser)
    }
}

pub fn make_rule_set(yaml_file: &YamlFile) -> Result<HashMap<Box<str>, Rule>, AnyBoxedError> {
    match &yaml_file.yaml.data {
        YamlData::Mapping(mapping) => {
            let mut rule_set = HashMap::new();
            for (key, value) in mapping {
                if let YamlData::String(rule_name) = &key.data {
                    let rule = Rule {
                        name: rule_name.clone().into(),
                        node: value.clone(),
                        is_used: false,
                    };
                    rule_set.insert(Box::from(rule_name.clone()), rule);
                }
            }
            Ok(rule_set)
        }
        _ => Err(SimpleError::from_message("The document should be a YAML map.".into()).into()),
    }
}
