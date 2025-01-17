use lidy__yaml::{Yaml, YamlData};

pub fn extract_kv_entry<'a>(entry_list: &'a Vec<(Yaml, Yaml)>, key: &str) -> Option<&'a Yaml> {
    entry_list
        .iter()
        .find_map(|(yaml_key, yaml_value)| match &yaml_key.data {
            YamlData::String(content) if content == key => Some(yaml_value),
            _ => None,
        })
}
