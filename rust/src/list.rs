use lidy__yaml::{Yaml, YamlData};

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::expression::apply_expression;
use crate::lidy::{Builder, Parser};
use crate::result::{Data, LidyResult, ListData};

pub fn apply_list_matcher<TV>(
    parser: &mut Parser<TV>,
    list: Option<&Yaml>,
    list_facultative: Option<&Yaml>,
    list_of: Option<&Yaml>,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone,
{
    if let YamlData::List(content_list) = &content.data {
        let mut data = ListData {
            list: Vec::new(),
            list_of: Vec::new(),
        };
        let mut join_error = JoinError::default();
        let mut offset = 0;

        // Process mandatory list items
        if let Some(list_yaml) = list {
            if let YamlData::List(list_items) = &list_yaml.data {
                for (index, schema) in list_items.iter().enumerate() {
                    if index >= content_list.len() {
                        join_error.add(
                            SimpleError::from_check("_list", "not enough entries", content).into(),
                        );
                        break;
                    }
                    let outcome = apply_expression(parser, schema, &content_list[index]);
                    match outcome {
                        Err(err) => join_error.add(err),
                        Ok(lidy_result) => data.list.push(lidy_result),
                    }
                }

                offset += list_items.len();
            } else {
                panic!("the metaschema must ensure that _list is associated with a list")
            }
        }

        // Process facultative list items
        if let Some(list_facultative_yaml) = list_facultative {
            if let YamlData::List(list_facultative_items) = &list_facultative_yaml.data {
                for (k, schema) in list_facultative_items.iter().enumerate() {
                    let index = offset + k;
                    if index >= content_list.len() {
                        break;
                    }
                    let outcome = apply_expression(parser, schema, &content_list[index]);
                    match outcome {
                        Err(err) => join_error.add(err),
                        Ok(lidy_result) => data.list.push(lidy_result),
                    }
                }
                offset += list_facultative_items.len();
            }
        }

        // Process list_of items
        if let Some(list_of_yaml) = list_of {
            for k in offset..content_list.len() {
                let outcome = apply_expression(parser, list_of_yaml, &content_list[k]);
                match outcome {
                    Err(err) => {
                        join_error.add(err);
                    }
                    Ok(lidy_result) => data.list_of.push(lidy_result),
                }
            }
        } else if offset < content_list.len() {
            return Err(SimpleError::from_message("Too many entries in list".into()).into());
        }

        join_error.into_result()?;

        Ok(LidyResult::create(&parser, &content, Data::ListData(data)))
    } else {
        Err(SimpleError::from_check("_list*", "must be a sequence node".into(), content).into())
    }
}
