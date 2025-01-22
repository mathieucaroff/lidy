use std::collections::HashMap;

use lidy__yaml::{LineCol, Yaml};

use crate::{
    error::AnyBoxedError, result::Data, rule::apply_predefined_rule, LidyResult, Parser,
    SimpleError,
};

impl<'a, TV> Parser<'a, TV> {
    pub fn run_rule_reference_checker_builder(
        &mut self,
        lidy_result: &LidyResult<()>,
    ) -> Result<Data<()>, AnyBoxedError> {
        let identifier = match &lidy_result.data {
            Data::String(s) => s.to_string(),
            _ => {
                panic!("never non-string identifier for rule reference");
                // return Ok(lidy_result.data.clone())
            }
        };

        if let Some(rule) = self.rule_set.get_mut(&*identifier) {
            rule.is_used = true;
        } else {
            let rule_exists = match apply_predefined_rule(
                &mut Parser{
                    content_file_name: "ruleCheck".into(),
                    rule_set: HashMap::new(),
                    rule_trace: Vec::new(),
                    rule_is_matching_node: HashMap::new(),
                    builder_callback: Box::new(
                        |_: &str,
                         _: &LidyResult<()>|
                         -> Result<Data<()>, Box<dyn std::error::Error>> {
                            panic!("never")
                        },
                    ),
                },
                &identifier,
                &Yaml::default(),
                true,
            )?.data {
                Data::Boolean(exists) => exists,
                _ => panic!("never, apply_predefined_rule must return a boolean when only_check_if_rule_exists is passed")
            };
            if !rule_exists {
                let rule_listing = self
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
                        line: lidy_result.position.line,
                        column: lidy_result.position.column,
                    },
                )));
            }
        }
        Ok(lidy_result.data.clone())
    }
}
