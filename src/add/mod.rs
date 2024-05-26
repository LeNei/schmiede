pub mod database;

use crate::config;
use anyhow::{Context, Result};
use askama::Template;
use clap::Subcommand;
use std::{fs, path::Path};
use toml_edit::{value, Array, DocumentMut};

//TODO: Update schmiede.toml
#[derive(Subcommand, Debug)]
pub enum Features {
    Database(config::Database),
}

pub trait AddFeature {
    fn add_feature(&self, path: &Path) -> Result<()>;
}

type ProcessFunction = fn(&mut Vec<&str>, usize) -> ();
type BeforeFunction = fn(&mut Vec<&str>) -> ();
type AfterFunction = fn(&mut Vec<&str>, Vec<bool>) -> ();

struct FileEditor<'a> {
    file_path: &'a Path,
    changes: Vec<(ProcessFunction, Vec<&'a str>)>,
    before_changes: Option<BeforeFunction>,
    after_changes: Option<AfterFunction>,
}

impl<'a> FileEditor<'a> {
    pub fn new(file_path: &'a Path) -> Self {
        Self {
            file_path,
            changes: Vec::new(),
            before_changes: None,
            after_changes: None,
        }
    }

    pub fn add_change(&mut self, change: ProcessFunction, conditions: Vec<&'a str>) -> &mut Self {
        self.changes.push((change, conditions));
        self
    }

    pub fn before_change(&mut self, change: BeforeFunction) -> &mut Self {
        self.before_changes = Some(change);
        self
    }

    pub fn after_change(&mut self, change: AfterFunction) -> &mut Self {
        self.after_changes = Some(change);
        self
    }

    pub fn edit_file(&self) -> Result<()> {
        let file = fs::read_to_string(self.file_path)
            .context(format!("Failed to read file: {}", self.file_path.display()))?;
        let mut lines = file.lines().collect::<Vec<_>>();

        if let Some(change) = self.before_changes {
            change(&mut lines);
        }

        let mut has_been_called = vec![false; self.changes.len()];

        'lines: for (i, line) in lines.clone().iter().enumerate() {
            for (j, (change, conditions)) in self.changes.iter().enumerate() {
                if !has_been_called[j] && conditions.iter().all(|c| line.contains(c)) {
                    change(&mut lines, i);
                    has_been_called[j] = true;
                    continue 'lines;
                }
            }
        }

        if let Some(change) = self.after_changes {
            change(&mut lines, has_been_called);
        }

        let updated_file = lines.join("\n");
        fs::write(self.file_path, updated_file).context(format!(
            "Failed to update file: {}",
            self.file_path.display()
        ))?;
        Ok(())
    }

    pub fn create_file(&self, content: &str) -> Result<()> {
        fs::write(self.file_path, content).context(format!(
            "Failed to create file: {}",
            self.file_path.display()
        ))
    }
}

pub type Dependency = (&'static str, &'static str, Option<Vec<&'static str>>);

pub fn add_dependencies(path: &Path, dependencies: Vec<Dependency>) -> Result<()> {
    let toml_path = path.join("Cargo.toml");
    let toml_contents =
        fs::read_to_string(&toml_path).with_context(|| "Failed to read Cargo.toml")?;

    let mut manifest = toml_contents
        .parse::<DocumentMut>()
        .with_context(|| "Failed to parse Cargo.toml")?;

    let deps = manifest
        .get_mut("dependencies")
        .ok_or(anyhow::anyhow!("Failed to get dependencies"))?;

    for (name, version, features) in dependencies {
        if let Some(features) = features {
            deps[name]["version"] = value(version);
            let mut array = Array::default();
            for feature in features {
                array.push(feature);
            }

            deps[name]["features"] = value(array);
        } else {
            deps[name] = value(version.to_string());
        }
    }

    let updated_toml = manifest.to_string();
    fs::write(toml_path, updated_toml).with_context(|| "Failed to write Cargo.toml")?;
    Ok(())
}

pub fn write_config<T: Template>(path: &Path, template: &T) -> Result<()> {
    let rendered = template
        .render()
        .with_context(|| "Failed to render template")?;
    fs::write(path, rendered)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(())
}
