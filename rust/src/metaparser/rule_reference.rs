use std::collections::HashMap;

use lidy__yaml::{LineCol, Yaml};

use crate::{
    error::AnyBoxedError, lidy::Builder, result::Data, rule::apply_predefined_rule, LidyResult,
    Parser, SimpleError,
};

impl<'a, TV> Builder<()> for RuleReferenceBuilder<'a, TV>
where
    TV: Clone,
{
    fn build(&self, input: &LidyResult<()>) -> Result<Data<()>, AnyBoxedError> {
        let identifier = match &input.data {
            Data::String(s) => s.to_string(),
            _ => return Ok(input.data.clone()),
        };

        if let Some(rule) = self.subparser.rule_set.get_mut(&*identifier) {
            rule.is_used = true;
        } else {
            let rule_exists = match apply_predefined_rule(
                &mut Parser{
                    content_file_name: "ruleCheck".into(),
                    rule_set: HashMap::new(),
                    builder_map: HashMap::<Box<str>, ()>::new(),
                    rule_trace: Vec::new(),
                    rule_is_matching_node: HashMap::new(),
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
                    .subparser
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
    }
}
