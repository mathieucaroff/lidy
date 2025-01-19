use std::rc::Rc;

use lidy__yaml::{Yaml, YamlLoader};

use crate::error::{AnyBoxedError, SimpleError};
use crate::file::File;

#[derive(Clone, Debug)]
pub struct YamlFile {
    pub file: Rc<File>,
    pub yaml: Yaml,
}

impl YamlFile {
    pub fn new(file: Rc<File>) -> Self {
        Self {
            file,
            yaml: Yaml::default(),
        }
    }

    pub fn deserialize(&mut self) -> Result<(), AnyBoxedError> {
        let docs = YamlLoader::load_from_str(&self.file.content)
            .map_err(|e| SimpleError::from_str(&e.to_string()))?;
        if docs.is_empty() {
            return Err("No YAML document found".into());
        }
        Ok(())
    }
}
