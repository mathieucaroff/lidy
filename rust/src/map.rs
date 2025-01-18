use lidy__yaml::{Yaml, YamlData};
use std::collections::HashMap;

use crate::error::{AnyBoxedError, JoinError, SimpleError};
use crate::expression::apply_expression;
use crate::lidy::{Builder, ParserData};
use crate::result::{Data, LidyResult, MapData};
use crate::syaml::extract_kv_entry;
use crate::KeyValueData;

struct MapInfo {
    mandatory_keys: HashMap<Box<str>, bool>,
    map: HashMap<Box<str>, Yaml>,
}

fn resolve_merge_reference<'a, T, TB>(
    parser_data: &'a ParserData<TV, TB>,
    node: &Yaml,
) -> Result<&'a Vec<(Yaml, Yaml)>, AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    match &node.data {
        YamlData::Mapping(yaml_mapping) => Ok(yaml_mapping),
        YamlData::String(ref rule_name) => {
            let rule = parser_data
                .parser
                .rule_set
                .get(&**rule_name)
                .ok_or_else(|| {
                    SimpleError::from_message(
                        "The merge value reference must exist in the schema".into(),
                    )
                })?;
            resolve_merge_reference(parser_data, &rule.node)
        }
        _ => Err(SimpleError::from_message(
            "The merge values must be mappings or references to mappings".into(),
        )
        .into()),
    }
}

fn contribute_to_map_info<TV, TB>(
    parser_data: &ParserData<TV, TB>,
    map_info: &mut MapInfo,
    map: Option<&Yaml>,
    map_facultative: Option<&Yaml>,
    merge: Option<&Yaml>,
) -> Result<(), AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    // Extracting from _merge
    if let Some(merge_yaml) = merge {
        if let YamlData::List(merge_list) = &merge_yaml.data {
            for node in merge_list {
                let resolved_vec: &Vec<(Yaml, Yaml)> = resolve_merge_reference(parser_data, node)?;
                let map_node = extract_kv_entry(resolved_vec, "_map");
                let map_facultative_node = extract_kv_entry(resolved_vec, "_mapFacultative");
                let merge_node = extract_kv_entry(resolved_vec, "_merge");
                contribute_to_map_info(
                    parser_data,
                    map_info,
                    map_node,
                    map_facultative_node,
                    merge_node,
                )?
            }
        }
    }

    // Extracting from _map
    if let Some(map_yaml) = map {
        if let YamlData::Mapping(mapping) = &map_yaml.data {
            for (key, value) in mapping {
                if let YamlData::String(key_string) = &key.data {
                    map_info
                        .map
                        .insert(key_string.clone().into(), value.clone());
                    map_info
                        .mandatory_keys
                        .insert(key_string.clone().into(), true);
                }
            }
        }
    }

    // Extracting from _mapFacultative
    if let Some(map_facultative_yaml) = map_facultative {
        if let YamlData::Mapping(mapping) = &map_facultative_yaml.data {
            for (key, value) in mapping {
                if let YamlData::String(key_string) = &key.data {
                    let key_string_box = key_string.clone().into();
                    if !map_info.mandatory_keys.contains_key(&key_string_box) {
                        map_info.map.insert(key_string_box, value.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn apply_map_matcher<TV, TB>(
    parser_data: &mut ParserData<TV, TB>,
    map: Option<&Yaml>,
    map_facultative: Option<&Yaml>,
    map_of: Option<&Yaml>,
    merge: Option<&Yaml>,
    content: &Yaml,
) -> Result<LidyResult<TV>, AnyBoxedError>
where
    TV: Clone,
    TB: Builder<TV>,
{
    if let YamlData::Mapping(content_mapping) = &content.data {
        let mut map_info = MapInfo {
            mandatory_keys: HashMap::new(),
            map: HashMap::new(),
        };

        contribute_to_map_info(parser_data, &mut map_info, map, map_facultative, merge)?;

        let mut map_data = MapData {
            map: HashMap::new(),
            map_of: Vec::new(),
        };
        let mut join_error = JoinError::default();

        let mut map_content = HashMap::<Box<str>, &Yaml>::new();
        for (key, value) in content_mapping {
            if let YamlData::String(key_string) = &key.data {
                map_content.insert(Box::from(key_string.as_str()), value);
            }
        }

        for key in map_info.mandatory_keys.keys() {
            if !map_content.contains_key(key) {
                join_error.add(
                    SimpleError::from_check(
                        "_map",
                        &format!("missing key '{key}' in mapping"),
                        content,
                    )
                    .into(),
                )
            }
        }

        for (key, value) in content_mapping {
            let mut unknown_key = true;
            if let YamlData::String(ref ks) = key.data {
                if let Some(schema) = map_info.map.get(&**ks) {
                    unknown_key = false;
                    match apply_expression(parser_data, schema, value) {
                        Ok(result) => {
                            map_data.map.insert(ks.clone().into(), result);
                        }
                        Err(e) => join_error
                            .add(SimpleError::from_message(format!("key {ks}: {e}").into()).into()),
                    }
                }
            }

            if unknown_key {
                match map_of {
                    Some(ref map_of_node) => match &map_of_node.data {
                        YamlData::Mapping(map_of_mapping) => {
                            let mut maybe_join_error = JoinError::default();
                            let association_count = map_of_mapping.len();
                            maybe_join_error.add(
                                SimpleError::from_check("_mapOf", &format!("none of the {association_count} _mapOf association(s) matched"), map_of_node).into());
                            let mut match_found = false;

                            for (schema_key, schema_value) in map_of_mapping {
                                // Key check
                                let key_outcome = apply_expression(parser_data, schema_key, key);
                                if let Err(ref key_error) = key_outcome {
                                    maybe_join_error.add(
                                        SimpleError::from_check(
                                            "_mapOf[key]",
                                            &key_error.to_string(),
                                            key,
                                        )
                                        .into(),
                                    )
                                }
                                // Value check
                                let value_outcome =
                                    apply_expression(parser_data, schema_value, value);
                                if let Err(ref value_error) = value_outcome {
                                    maybe_join_error.add(
                                        SimpleError::from_check(
                                            "_mapOf[value]",
                                            &value_error.to_string(),
                                            value,
                                        )
                                        .into(),
                                    )
                                }
                                // Match if both the key and value check passed
                                if key_outcome.is_ok() && value_outcome.is_ok() {
                                    map_data.map_of.push(KeyValueData {
                                        key: key_outcome.ok().unwrap(),
                                        value: value_outcome.ok().unwrap(),
                                    });
                                    match_found = true;
                                    break;
                                }
                            }
                            if !match_found {
                                join_error.add(maybe_join_error.into())
                            }
                        }
                        _ => {
                            panic!("_mapOf associations must be a yaml mapping")
                        }
                    },
                    None => {
                        let mut error_description =
                            "unrecognized non-string non-integer key in mapping".to_string();
                        if let YamlData::String(ref ks) = key.data {
                            error_description = format!("unknown key '{ks}' in mapping")
                        } else if let YamlData::Integer(kn) = key.data {
                            error_description = format!("unknown numeric key '{kn}' in mapping")
                        }
                        join_error
                            .add(SimpleError::from_check("_map*", &error_description, key).into())
                    }
                }
            }
        }

        join_error.into_result()?;

        Ok(LidyResult::create(
            parser_data,
            content,
            Data::MapData(map_data),
        ))
    } else {
        Err(SimpleError::from_check("_map*", "must be a mapping node", content).into())
    }
}
