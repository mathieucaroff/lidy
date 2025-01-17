use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lidy__yaml::{LineCol, Yaml, YamlData};

use crate::error::{AnyBoxedError, SimpleError};
use crate::file::File;
use crate::metaparser::{check_rule_set, make_meta_parser_for};
use crate::result::{Data, LidyResult};
use crate::rule::{apply_rule, Rule};
use crate::yamlfile::YamlFile;

pub type BuilderFn<T> = Box<dyn FnMut(&LidyResult<T>) -> Result<Data<T>, AnyBoxedError>>;
pub type Builder<T> = Rc<RefCell<BuilderFn<T>>>;

#[derive(Clone, Default)]
pub struct Parser<T>
where
    T: Clone,
{
    pub rule_set: HashMap<Box<str>, Rule<T>>,
}

#[derive(Clone, Default)]
pub struct ParserData<T>
where
    T: Clone + 'static,
{
    pub content_file_name: Rc<str>,
    pub parser: Parser<T>,
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

impl<T> Parser<T>
where
    T: Clone + 'static,
{
    pub fn parse(&self, content_file: &Rc<File>) -> Result<LidyResult<T>, AnyBoxedError> {
        let mut yaml_content_file = YamlFile::new(content_file.clone());
        yaml_content_file.unmarshal()?;
        self._parse_data(&yaml_content_file)
    }

    fn _parse_data(&self, content: &YamlFile) -> Result<LidyResult<T>, AnyBoxedError> {
        let mut parser_data = ParserData {
            parser: self.clone(),
            content_file_name: content.file.name.clone().into(),
            rule_is_matching_node: HashMap::new(),
            rule_trace: Vec::new(),
        };

        return apply_rule(&mut parser_data, "main", &content.yaml);
    }

    pub fn make(
        file: &Rc<File>,
        builder_map: HashMap<Box<str>, Builder<T>>,
    ) -> Result<Parser<T>, AnyBoxedError>
    where
        T: Clone,
    {
        let mut schema_file = YamlFile::new(file.clone());
        schema_file.unmarshal()?;

        let rule_set = make_rule_set(&schema_file, builder_map)?;
        let mut parser = Parser { rule_set };

        // METAPARSING VALIDATION
        // Validate that the provided schema is valid according to the lidy metaschema
        let meta_parser = make_meta_parser_for(&mut parser)?;
        meta_parser._parse_data(&schema_file)?;
        check_rule_set(&mut parser.rule_set);

        Ok(parser)
    }
}

pub fn make_rule_set<T>(
    yaml_file: &YamlFile,
    mut builder_map: HashMap<Box<str>, Builder<T>>,
) -> Result<HashMap<Box<str>, Rule<T>>, AnyBoxedError>
where
    T: Clone,
{
    match &yaml_file.yaml.data {
        YamlData::Mapping(mapping) => {
            let mut rule_set = HashMap::new();
            for (key, value) in mapping {
                if let YamlData::String(rule_name) = &key.data {
                    let builder = builder_map.remove(&**rule_name);
                    let rule = Rule {
                        name: rule_name.clone().into(),
                        node: value.clone(),
                        builder,
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
