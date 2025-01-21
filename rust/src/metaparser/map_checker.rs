use lidy__yaml::{LineCol, YamlData};

use crate::{
    builder::BuilderTrait,
    error::{AnyBoxedError, JoinError},
    result::Data,
    LidyResult, Parser, Position, SimpleError,
};

#[derive(Clone)]
pub struct MapCheckerBuilder;

impl<TV> BuilderTrait<MapCheckerBuilder> for Parser<TV>
where
{
    fn build(
        &mut self,
        lidy_result: &LidyResult<MapCheckerBuilder>,
    ) -> Result<Data<MapCheckerBuilder>, AnyBoxedError> {
        if let Data::MapData(map_data) = &lidy_result.data {
            if let Some(merge) = map_data.map.get("_merge") {
                let mut join_error = JoinError::default();
                if let Data::ListData(list_data) = &merge.data {
                    for result in &list_data.list_of {
                        match &result.data {
                            Data::CustomData(_) => continue,
                            Data::String(s) => {
                                if let Some(err) =
                                    self.check_merged_node(s, &result.position, &result.position)
                                {
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
        Ok(lidy_result.data.clone())
    }
}

impl<TV> Parser<TV>
where
{
    fn check_merged_node(
        &self,
        name: &str,
        last_position: &Position,
        origin_position: &Position,
    ) -> Option<AnyBoxedError> {
        let rule = self.rule_set.get(name);
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
            YamlData::String(name) => self.check_merged_node(name, &last_pos, origin_position),
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
}
