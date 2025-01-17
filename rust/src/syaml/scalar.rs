use lidy__yaml::{Yaml, YamlData};

pub fn is_scalar(node: &Yaml) -> bool {
    match &node.data {
        YamlData::Real(_) => true,
        YamlData::Integer(_) => true,
        YamlData::String(_) => true,
        YamlData::Boolean(_) => true,
        YamlData::Null => true,
        _ => false,
    }
}
