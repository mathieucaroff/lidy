use std::fmt;

use lidy__yaml::{LineCol, Yaml};

pub type AnyBoxedError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct SimpleError {
    message: Box<str>,
}

impl SimpleError {
    pub fn from_str(text: &str) -> SimpleError {
        SimpleError::from_message(text.into())
    }
    pub fn from_message(message: Box<str>) -> SimpleError {
        SimpleError { message }
    }
    pub fn from_check(keyword: &str, description: &str, node: &Yaml) -> SimpleError {
        let line_col = node.line_col;
        Self::from_message(format!("{keyword}: {description} {line_col}").into())
    }
    pub fn from_check_result(rule_name: &str, description: &str, line_col: LineCol) -> SimpleError {
        Self::from_message(format!("{rule_name}: {description} {line_col}").into())
    }
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SimpleError {}

#[derive(Debug, Default)]

pub struct JoinError {
    errors: Vec<Box<dyn std::error::Error>>,
}

impl JoinError {
    pub fn add(&mut self, error: Box<dyn std::error::Error>) {
        self.errors.push(error)
    }
    pub fn into_result(self) -> Result<(), Box<dyn std::error::Error>> {
        if self.errors.len() == 0 {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

impl fmt::Display for JoinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string_vec = vec![];

        for err in &self.errors {
            string_vec.push(err.to_string());
        }

        write!(f, "{}", string_vec.join("; "))
    }
}

impl std::error::Error for JoinError {}
