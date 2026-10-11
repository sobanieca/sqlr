//! Temporary stub, replaced by the Cliffy command port.
#![allow(dead_code)]

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub arguments: &'static str,
    pub options: Vec<Opt>,
    pub meta: Vec<(&'static str, &'static str)>,
    pub commands: Vec<Command>,
}

pub struct Opt {
    pub flags: &'static str,
    pub description: &'static str,
    pub conflicts: &'static [&'static str],
    pub collect: bool,
    pub global: bool,
    pub values: Option<Vec<String>>,
}

pub enum Value {
    Bool(bool),
    String(String),
    List(Vec<String>),
}

pub struct Matches {
    pub command: Vec<String>,
    pub options: Vec<(String, Value)>,
    pub args: Vec<Value>,
}

impl Matches {
    pub fn flag(&self, _name: &str) -> bool {
        todo!()
    }
    pub fn string(&self, _name: &str) -> Option<String> {
        todo!()
    }
    pub fn is_set(&self, _name: &str) -> bool {
        todo!()
    }
    pub fn list(&self, _name: &str) -> Vec<String> {
        todo!()
    }
    pub fn arg(&self, _index: usize) -> Option<&Value> {
        todo!()
    }
}

pub enum Outcome {
    Run(Matches),
    Exit(i32),
}

pub fn run(_root: &Command, _version: &str, _args: &[String]) -> Outcome {
    todo!()
}

pub fn help(_root: &Command, _version: &str, _path: &[&str]) -> String {
    todo!()
}
