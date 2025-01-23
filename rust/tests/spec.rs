use std::rc::Rc;

use regex::Regex;
use specimen;

fn template_read_entry(tile: &specimen::Dict, key: &str) -> Option<Box<str>> {
    let value = tile.get(key);
    let template_value = tile.get(&format!("{key}Template").into_boxed_str());
    if value.is_some() && template_value.is_some() {
        panic!("Found both '{key}' and '{key}Template'. Only one must be specified.");
    }
    if let Some(&ref v) = value {
        Some(v.clone())
    } else if let Some(template_v) = template_value {
        let result_value = Regex::new(r"\$\{([a-zA-Z0-9_]+)\}").unwrap().replace_all(
            &template_v,
            |m: &regex::Captures| {
                let text: Box<str> = m.get(0).unwrap().as_str().into();
                let name: Box<str> = text[2..text.len() - 1].into();
                let value = tile.get(&name).expect(&format!(
                    "the template interpolation key '{name}' was not found in the input"
                ));
                value
            },
        );
        Some(result_value.into())
    } else {
        None
    }
}

#[test]
fn test_lidy() {
    let test_passed = specimen::run(
        &mut |tile: &specimen::Dict| -> Result<(), Box<str>> {
            // Target
            let box_ = match tile.get("box") {
                Some(v) => &v,
                None => "content",
            };

            // Unpacking the input data

            // Text
            let text = template_read_entry(tile, "text");
            if text.is_none() {
                return Err(Box::from("The 'text' entry is required"));
            }

            // Expression and Schema
            let expression = template_read_entry(tile, "expression");
            let mut schema = template_read_entry(tile, "schema");
            // checking the expression and schema values depending on the box
            if box_ == "content" {
                if expression.is_some() && schema.is_some() {
                    return Err(Box::from(
                        "'expression' and 'schema' cannot be specified together",
                    ));
                }
                if expression.is_none() && schema.is_none() {
                    return Err(Box::from(
                        "one of 'expression' and 'schema' must be specified",
                    ));
                }
                if let Some(exp) = expression {
                    schema = Some(format!("main:\n  {}", exp.replace("\n", "\n  ")).into());
                }
            } else {
                let mut word = "";
                if expression.is_some() {
                    word = "expression";
                } else if schema.is_some() {
                    word = "schema";
                }
                if word != "" {
                    return Err(format!("box {box_} should not receive any {word}").into());
                }
            }

            // Name and Outcome
            let name = tile.get("name").unwrap();
            let mut outcome = "";
            if name.starts_with("accept") {
                outcome = "accept";
            } else if name.starts_with("reject") {
                outcome = "reject";
            }
            if outcome == "" {
                return Err(format!(
                    "the 'name' entry should begin by 'accept' or 'reject', but it is: {name}"
                )
                .into());
            }

            // // // // // // // // // // // // // // // // // // // // //
            let mut error: Option<Box<str>> = None;
            if box_ == "content" {
                if schema.is_none() {
                    return Err(Box::from("the schema cannot be empty"));
                }
                // TODO check this call
                let parser = lidy::Parser::<()>::make(
                    &Rc::from(lidy::File {
                        name: "<schema>.yaml".into(),
                        content: schema.unwrap(),
                    }),
                    Box::new(|_, lidy_result| Ok(lidy_result.data.clone())),
                );
                if parser.is_err() {
                    return Err(Box::from(format!(
                        "error in schema: {}",
                        parser.err().unwrap()
                    )));
                }

                let file = lidy::File {
                    name: "<content>.yaml".into(),
                    content: text.unwrap(),
                };

                if let Err(err) = parser.unwrap().parse(&Rc::from(file)) {
                    error = Some(format!("error in content: {}", err).into());
                }
            } else {
                match box_ {
                    "lidySchemaExpression" => {
                        schema = Some(
                            format!("main:\n  {}", text.unwrap().replace("\n", "\n  ")).into(),
                        );
                    }
                    "lidySchemaDocument" => {
                        schema = text;
                    }
                    "lidySchemaRegexChecker" => {
                        schema = Some(format!("main:\n  _regex: '{}'", text.unwrap()).into());
                    }
                    _ => {
                        return Err(Box::from(format!("unknown test box: {}", box_)));
                    }
                }
                // TODO check this call once make_parser is implemented
                let parser_result = lidy::Parser::<()>::make(
                    &Rc::from(lidy::File {
                        name: "<schema>.yaml".into(),
                        content: schema.unwrap().clone(),
                    }),
                    Box::new(|_, lidy_result| Ok(lidy_result.data.clone())),
                );

                println!("PARSER RESULT: {:?}", parser_result);

                if let Err(err) = parser_result {
                    error = Some(err.to_string().into());
                }
            }

            // // // // // // // // // // // // // // // // // // // // //

            let error_contains = tile.get("errorContains");

            if outcome == "accept" {
                if error_contains.is_some() {
                    return Err(
                        "'errorContains' cannot be specified when the expected outcome is 'accept'"
                            .into(),
                    );
                }
                if let Some(err) = error {
                    return Err(format!("error: {err}").into());
                }
            } else {
                if error.is_none() {
                    return Err(format!("no error was found ({name})").into());
                }
                if let Some(wanted) = error_contains {
                    let message = error.unwrap();
                    if !message.contains(&wanted.to_string()) {
                        return Err(format!(
                            "error message '{message}' does not contain '{wanted}'"
                        )
                        .into());
                    }
                }
            }

            Ok(())
        },
        &[
            specimen::file::File::read_local_file("../testdata/collection/listOf.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/collection/map.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/collection/mapOf.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/collection/merge.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/collection/min_max_nb.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/collection/tuple.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/combinator/oneOf.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/scalar/in.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/scalar/range.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/scalar/regexp.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/scalarType/scalar.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/schema/document.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/schema/expression.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/schema/mergeChecker.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/schema/regex.spec.yaml"),
            specimen::file::File::read_local_file("../testdata/yaml/yaml.spec.yaml"),
        ],
    );

    assert!(test_passed);
}
