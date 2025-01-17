use lidy__yaml::{Yaml, YamlData};

use crate::{error::AnyBoxedError, SimpleError};

pub fn apply_size_check(
    content: &Yaml,
    min: Option<&Yaml>,
    max: Option<&Yaml>,
    nb: Option<&Yaml>,
) -> Option<AnyBoxedError> {
    // Get the length of items if it's a sequence or mapping
    let size = match &content.data {
        YamlData::List(seq) => seq.len(),
        YamlData::Mapping(map) => map.len(),
        _ => {
            panic!("_(size), Only containers (maps or sequences) have a size.");
        }
    };

    // Check minimum size if specified
    if let Some(min_node) = min {
        if let YamlData::Integer(min_size) = &min_node.data {
            if size < *min_size as usize {
                return Some(
                    SimpleError::from_check(
                        "_min",
                        &format!(
                            "Expected container to have at least {} entries but it has only {}.",
                            min_size, size
                        ),
                        content,
                    )
                    .into(),
                );
            }
        }
    }

    // Check maximum size if specified
    if let Some(max_node) = max {
        if let YamlData::Integer(max_size) = &max_node.data {
            if size > *max_size as usize {
                return Some(
                    SimpleError::from_check(
                        "_max",
                        &format!(
                            "Expected container to have at most {} entries but it has {}.",
                            max_size, size
                        ),
                        content,
                    )
                    .into(),
                );
            }
        }
    }

    // Check exact size if specified
    if let Some(nb_node) = nb {
        if let YamlData::Integer(nb_size) = &nb_node.data {
            if size != *nb_size as usize {
                return Some(
                    SimpleError::from_check(
                        "_nb",
                        &format!(
                            "Expected container to have exactly {} entries but it has {}.",
                            nb_size, size
                        ),
                        content,
                    )
                    .into(),
                );
            }
        }
    }

    None
}
