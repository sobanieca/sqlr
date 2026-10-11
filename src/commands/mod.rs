//! Command implementations.

pub mod add;
pub mod clear;
pub mod describe;
pub mod get;
pub mod ls;
pub mod ls_types;
pub mod query;
pub mod rm;
pub mod set;
pub mod set_global;
pub mod skill;

pub const MAX_TABLE_COLUMN_WIDTH: usize = 80;

pub const HELP_TEXT: &str = include_str!("../help.txt");
