use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::File;


const TABLES_FILE: &str = "tables.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Method {
    pub future: bool,
    pub name: String,
    pub args: Vec<String>,
    pub generics: String,
    pub outputs: String,
}


fn read_tables() -> std::io::Result<Option<HashMap<String, String>>> {
    if !std::path::Path::new(TABLES_FILE).exists() {
        return Ok(None);
    }
    let mut file = File::open(TABLES_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(Some(serde_json::from_str(&contents)?))
}


fn write_tables(tables: &HashMap<String, String>) -> std::io::Result<()> {
    let json = serde_json::to_string(tables)?;
    let mut file = File::create(TABLES_FILE)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn add_table(trait_name: String, value: String) {
    let mut all_tables = HashMap::new();
    if let Ok(Some(contents)) = read_tables() {
        all_tables = contents;
    }
    all_tables.insert(trait_name, value);
    write_tables(&all_tables).expect("could not write tables to a file")
}

pub fn get_tables() -> HashMap<String, String> {
    if let Some(contents) = read_tables().unwrap() {
        return contents;
    }
    HashMap::new()
}

pub fn delete_tables_file() -> std::io::Result<()> {
    if std::path::Path::new(TABLES_FILE).exists() {
        std::fs::remove_file(TABLES_FILE)?;
    }
    Ok(())
}