use std::fs;

use crate::error::{AnyBoxedError, SimpleError};

#[derive(Clone, Debug)]
pub struct File {
    pub name: Box<str>,
    pub content: Box<str>,
}

impl File {
    pub fn read_local_file(path: &str) -> Result<File, AnyBoxedError> {
        let content =
            fs::read_to_string(path).map_err(|e| SimpleError::from_str(&e.to_string()))?;
        Ok(File {
            name: path.into(),
            content: content.into(),
        })
    }
}
