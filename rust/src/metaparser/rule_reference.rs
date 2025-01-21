use std::collections::HashMap;

use lidy__yaml::{LineCol, Yaml};

use crate::{
    builder::BuilderTrait, error::AnyBoxedError, result::Data, rule::apply_predefined_rule,
    LidyResult, Parser, SimpleError,
};

#[derive(Clone)]
pub struct RuleReferenceBuilder;

impl<TV> BuilderTrait<RuleReferenceBuilder> for Parser<TV>
where
{
    fn build(
        &mut self,
        lidy_result: &LidyResult<RuleReferenceBuilder>,
    ) -> Result<Data<RuleReferenceBuilder>, AnyBoxedError> {
        let identifier = match &lidy_result.data {
            Data::String(s) => s.to_string(),
            _ => return Ok(lidy_result.data.clone()),
        };

        if let Some(rule) = self.rule_set.get_mut(&*identifier) {
            rule.is_used = true;
        } else {
            let rule_exists = match apply_predefined_rule(
                &mut Parser{
                    content_file_name: "ruleCheck".into(),
                    rule_set: HashMap::new(),
                    builder_map: HashMap::<Box<str>, Box<dyn BuilderTrait<TV>>>::new(),
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
