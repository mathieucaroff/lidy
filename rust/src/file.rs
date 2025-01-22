use std::fs;

use crate::error::AnyBoxedError;

#[derive(Clone, Debug)]
pub struct File {
    pub name: Box<str>,
    pub content: Box<str>,
}

impl File {
    pub fn read_local_file(path: &str) -> Result<File, AnyBoxedError> {
        match fs::read_to_string(path) {
            Ok(content) => Ok(File {
                name: path.into(),
                content: content.into(),
            }),
            Err(e) => Err(format!("Failed to read file {path}: {e}").into()),
        }
    }
}
