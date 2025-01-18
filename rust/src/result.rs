use std::{collections::HashMap, rc::Rc};

use lidy__yaml::{LineCol, Yaml};

use crate::lidy::{Builder, ParserData};

#[derive(Clone, Debug, Default)]
pub struct Position {
    pub filename: Rc<str>,
    pub line: usize,
    pub column: usize,
    pub line_end: usize,
    pub column_end: usize,
}

impl Position {
    pub fn from_line_col_beginning_only(filename: Rc<str>, line_col: LineCol) -> Position {
        Position {
            filename,
            line: line_col.line,
            column: line_col.column,
            line_end: line_col.line,
            column_end: line_col.column,
        }
    }
}

impl From<Position> for LineCol {
    fn from(position: Position) -> Self {
        LineCol::from(&position)
    }
}

impl From<&Position> for LineCol {
    fn from(position: &Position) -> Self {
        LineCol {
            line: position.line,
            column: position.column,
        }
    }
}

impl<TV> From<&LidyResult<TV>> for LineCol
where
    TV: Clone,
{
    fn from(result: &LidyResult<TV>) -> Self {
        LineCol {
            line: result.position.line,
            column: result.position.column,
        }
    }
}

/// A result of an expression application. It has two generic types because
/// the data can be a mapping type, which itself requiers a key type and a
/// value type.
#[derive(Clone, Debug)]
pub struct LidyResult<TValue>
where
    TValue: Clone,
{
    pub position: Position,
    pub rule_name: Box<str>,
    pub data: Data<TValue>,
}

impl<TV> LidyResult<TV>
where
    TV: Clone,
{
    pub fn make(rule_name: &str, position: Position, data: Data<TV>) -> LidyResult<TV> {
        LidyResult::<TV> {
            position,
            rule_name: rule_name.into(),
            data,
        }
    }
    pub fn create<TB>(
        parser_data: &ParserData<TV, TB>,
        content: &Yaml,
        data: Data<TV>,
    ) -> LidyResult<TV>
    where
        TB: Builder<TV>,
    {
        LidyResult::<TV> {
            position: Position::from_line_col_beginning_only(
                parser_data.content_file_name.clone(),
                content.line_col,
            ),
            rule_name: parser_data.rule_trace.last().unwrap().clone(),
            data,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MapData<TValue>
where
    TValue: Clone,
{
    pub map: HashMap<Box<str>, LidyResult<TValue>>,
    pub map_of: Vec<KeyValueData<TValue>>,
}

#[derive(Clone, Debug)]
pub struct KeyValueData<TValue>
where
    TValue: Clone,
{
    pub key: LidyResult<TValue>,
    pub value: LidyResult<TValue>,
}

#[derive(Clone, Debug)]
pub struct ListData<TValue>
where
    TValue: Clone,
{
    pub list: Vec<LidyResult<TValue>>,
    pub list_of: Vec<LidyResult<TValue>>,
}

#[derive(Clone, Debug)]
pub enum Data<TValue>
where
    TValue: Clone,
{
    Float(f64),
    Integer(i64),
    String(Box<str>),
    Boolean(bool),
    Null,
    MapData(MapData<TValue>),
    ListData(ListData<TValue>),
    CustomData(TValue),
}
