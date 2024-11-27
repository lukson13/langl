use anyhow::{anyhow, Result};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    fs,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

#[derive(Debug)]
pub struct Collection {
    pub id: usize,
    lang: String,
    variables: HashMap<String, String>,
    words: HashMap<String, Vec<String>>,
}
impl PartialEq for Collection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Collection {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            lang: String::new(),
            variables: HashMap::new(),
            words: HashMap::new(),
        }
    }

    pub fn new_from_path(file: PathBuf, id: usize) -> Result<Self> {
        let mut file = fs::File::open(file)?;
        let mut s = String::new();
        file.read_to_string(&mut s);

        let mut coll = Collection::new(id);
        for (line_num, line) in s.split('\n').enumerate() {
            let linec = line.trim().chars().collect::<Vec<char>>();
            if linec.is_empty() || linec[0] == '#' {
                continue;
            }
            if linec[0] == '@' {
                let mut lang = String::new();
                for c in &linec[1..] {
                    lang.push(*c);
                }
                coll.lang = lang.trim().to_owned();
                continue;
            }
            if linec[0] == '$' {
                let mut name = String::new();
                let mut value = String::new();
                let mut after_eq = false;
                for c in &linec[1..] {
                    if !after_eq && *c == '=' {
                        after_eq = true;
                        continue;
                    }
                    if after_eq {
                        value.push(*c)
                    } else {
                        name.push(*c)
                    }
                }
                let name = name.trim();
                let value = value.trim();
                let old = coll.variables.insert(name.to_owned(), value.to_owned());
                if let Some(old) = old {
                    println!(
                        "variable changed | Line: {} | [ ({name}) {old} => {value} ]",
                        line_num + 1
                    );
                }
                continue;
            }

            let mut word = String::new();
            let mut meaning = String::new();
            let mut meanings = Vec::new();
            let mut after_pipe = false;

            for c in &linec {
                if *c == '|' {
                    if !after_pipe {
                        after_pipe = true;
                        continue;
                    } else {
                        println!("invalide use of next '|' |Line: {}|", line_num + 1)
                    }
                }
                if after_pipe {
                    if *c == '/' {
                        if !meaning.is_empty() {
                            meanings.push(meaning.trim().to_owned());
                            meaning = String::new();
                        }
                        continue;
                    }
                    meaning.push(*c)
                } else {
                    word.push(*c);
                }
            }
            if !meanings.is_empty() {
                meanings.push(meaning.trim().to_owned());
            }
            let word = word.trim();
            let old = coll.words.insert(word.to_owned(), meanings.clone());
            if let Some(old) = old {
                println!(
                    "word definition changed | Line: {} | [ ({word}) {old:?} => {meanings:?} ]",
                    line_num + 1
                );
            }
        }
        Ok(coll)
    }

    pub fn words(&self) -> &HashMap<String, Vec<String>> {
        &self.words
    }
}
impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} - {}",
            self.lang.as_str(),
            self.variables
                .get("name")
                .map_or("you fuck did not add name variable", |v| v)
        ))
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new(0)
    }
}
