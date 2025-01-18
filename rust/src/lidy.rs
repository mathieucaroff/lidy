use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use lidy__yaml::{LineCol, Yaml, YamlData};

use crate::error::{AnyBoxedError, SimpleError};
use crate::file::File;
use crate::metaparser::{check_rule_set, make_meta_parser_for};
use crate::result::{Data, LidyResult};
use crate::rule::{apply_rule, Rule};
use crate::yamlfile::YamlFile;

pub trait Builder<TV>
where
    TV: Clone,
{
    fn build(&self, rule_name: &str, content: &LidyResult<TV>) -> Result<Data<TV>, AnyBoxedError>;
}

#[derive(Clone, Default)]
pub struct Parser<TV, TB>
where
    TV: Clone,
    TB: Builder<TV>,
{
    _t: PhantomData<TV>,
    pub rule_set: HashMap<Box<str>, Rule>,
    pub builder: TB,
}

#[derive(Clone, Default)]
pub struct ParserData<TV, TB>
where
    TV: Clone + 'static,
    TB: Builder<TV>,
{
    pub content_file_name: Rc<str>,
    pub parser: Parser<TV, TB>,
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

impl<TV, TB> Parser<TV, TB>
where
    TV: Clone,
    TB: Builder<TV>,
{
    pub fn parse(&self, content_file: &Rc<File>) -> Result<LidyResult<TV>, AnyBoxedError> {
        let mut yaml_content_file = YamlFile::new(content_file.clone());
        yaml_content_file.unmarshal()?;
        self._parse_data(&yaml_content_file)
    }

    fn _parse_data(&self, content: &YamlFile) -> Result<LidyResult<TV>, AnyBoxedError> {
        let mut parser_data = ParserData::<TV, TB> {
            content_file_name: content.file.name.clone().into(),
            parser: self,
            rule_is_matching_node: HashMap::new(),
            rule_trace: Vec::new(),
        };

        return apply_rule(&mut parser_data, "main", &content.yaml);
    }

    pub fn make(file: &Rc<File>, builder: TB) -> Result<Self, AnyBoxedError> {
        let mut schema_file = YamlFile::new(file.clone());
        schema_file.unmarshal()?;

        let rule_set = make_rule_set(&schema_file)?;
        let mut parser = Parser {
            _t: PhantomData,
            rule_set,
            builder,
        };

        // METAPARSING VALIDATION
        // Validate that the provided schema is valid according to the lidy metaschema
        let meta_parser = make_meta_parser_for::<TV, TB>(&mut parser)?;
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
