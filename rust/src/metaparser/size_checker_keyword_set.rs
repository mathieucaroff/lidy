use lidy__yaml::LineCol;

use crate::{error::AnyBoxedError, result::Data, LidyResult, Parser, SimpleError};

impl<'a, TV> Parser<'a, TV> {
    pub fn run_size_checker_builder(
        &mut self,
        lidy_result: &LidyResult<()>,
    ) -> Result<Data<()>, AnyBoxedError> {
        if let Data::MapData(map_data) = &lidy_result.data {
            for keyword in &["_min", "_max", "_nb"] {
                if let Some(value) = map_data.map.get(*keyword) {
                    if let Data::Float(n) = &value.data {
                        if *n < 0.0 {
                            return Err(Box::new(SimpleError::from_check_result(
                                keyword,
                                "cannot be negative",
                                LineCol {
                                    line: lidy_result.position.line,
                                    column: lidy_result.position.column,
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
                    &format!("it makes no sense to use the `_nb` and `_{min_or_max}` together"),
                    LineCol {
                        line: lidy_result.position.line,
                        column: lidy_result.position.column,
                    },
                )));
            }

            if let (Some(min), Some(max)) = (min, max) {
                if min > max {
                    return Err(Box::new(SimpleError::from_check_result(
                        "_min",
                        "`_max` cannot be lower than `_min`",
                        LineCol {
                            line: lidy_result.position.line,
                            column: lidy_result.position.column,
                        },
                    )));
                }
            }
        }
        Ok(lidy_result.data.clone())
    }
}
