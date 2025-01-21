//! any.rs
//! Used in the anyData predefined rule as well as in the `_in` matcher

use std::{collections::HashMap, rc::Rc};

use lidy__yaml::{Yaml, YamlData};

use crate::{result::Data, KeyValueData, LidyResult, ListData, MapData, Position};

// Info:
//

pub fn map_any_yaml_data_to_lidy_data<TV>(
    filename: &Rc<str>,
    rule_name: &str,
    content: &Yaml,
) -> Data<TV>
where
{
    match &content.data {
        YamlData::String(value) => Data::String(value.clone().into()),
        YamlData::Integer(value) => Data::Integer(*value),
        YamlData::Real(value) => Data::Float(value.parse().ok().unwrap()),
        YamlData::Boolean(value) => Data::Boolean(*value),
        YamlData::Null => Data::Null,
        YamlData::Mapping(value_mapping) => {
            let mut map_data = MapData {
                map: HashMap::new(),
                map_of: Vec::new(),
            };
            for (key, value) in value_mapping {
                map_data.map_of.push(KeyValueData::<TV> {
                    key: LidyResult::make(
                        rule_name,
                        Position::from_line_col_beginning_only(filename.clone(), key.line_col),
                        map_any_yaml_data_to_lidy_data(filename, rule_name, key),
                    ),
                    value: LidyResult::make(
                        rule_name,
                        Position::from_line_col_beginning_only(filename.clone(), value.line_col),
                        map_any_yaml_data_to_lidy_data(filename, rule_name, value),
                    ),
                });
            }
            Data::MapData(map_data)
        }
        YamlData::List(list) => {
            let mut list_data = ListData {
                list: Vec::new(),
                list_of: Vec::new(),
            };
            for item in list {
                list_data.list_of.push(LidyResult::make(
                    rule_name,
                    Position::from_line_col_beginning_only(filename.clone(), item.line_col),
                    map_any_yaml_data_to_lidy_data(filename, rule_name, item),
                ))
            }
            Data::ListData(list_data)
        }
        YamlData::Alias(alias) => {
            panic!("alias not supported ({alias})");
        }
        YamlData::BadValue => {
            panic!("BadValue YAML value encountered");
        }
    }
}
