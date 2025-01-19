
let sized_checker_keyword_set_builder: Builder<TV> = {
    let builder_fn = Box::new(
        |input: &LidyResult<TV>| -> Result<Data<TV>, AnyBoxedError> {
            if let Data::MapData(map_data) = &input.data {
                for keyword in &["_min", "_max", "_nb"] {
                    if let Some(value) = map_data.map.get(*keyword) {
                        if let Data::Float(n) = &value.data {
                            if *n < 0.0 {
                                return Err(Box::new(SimpleError::from_check_result(
                                    keyword,
                                    "cannot be negative",
                                    LineCol {
                                        line: input.position.line,
                                        column: input.position.column,
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
                        &format!(
                            "it makes no sense to use the `_nb` and `_{min_or_max}` together"
                        ),
                        LineCol {
                            line: input.position.line,
                            column: input.position.column,
                        },
                    )));
                }

                if let (Some(min), Some(max)) = (min, max) {
                    if min > max {
                        return Err(Box::new(SimpleError::from_check_result(
                            "_min",
                            "`_max` cannot be lower than `_min`",
                            LineCol {
                                line: input.position.line,
                                column: input.position.column,
                            },
                        )));
                    }
                }
            }
            Ok(input.data.clone())
        },
    )
        as Box<dyn FnMut(&LidyResult<TV>) -> Result<Data<TV>, Box<(dyn std::error::Error)>>>;
    Rc::new(RefCell::new(builder_fn))
};
