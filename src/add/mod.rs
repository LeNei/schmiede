pub mod database;

use anyhow::{Context, Result};
use std::{fs, path::Path};

pub trait AddFeature {
    fn add_feature(&self, path: &Path) -> Result<()>;
}

type ProcessFunction = fn(&mut Vec<&str>, usize) -> ();
type AfterFunction = fn(&mut Vec<&str>, Vec<bool>) -> ();

struct FileEditor<'a> {
    file_path: &'a Path,
    changes: Vec<(ProcessFunction, Vec<&'a str>)>,
    before_changes: Option<ProcessFunction>,
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

    pub fn before_change(&mut self, change: ProcessFunction) -> &mut Self {
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
            change(&mut lines, 0);
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
}
