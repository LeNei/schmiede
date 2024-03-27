use crate::template::PageTemplate;

use super::template::{DbDownTemplate, DbUpTemplate, ModelTemplate};
use anyhow::Result;
use askama::Template;
use chrono::Utc;
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;

pub trait Export {
    fn export(&self) -> Result<()>;
}

impl Export for ModelTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        let file_path = String::from("src/common/models.rs");

        // Read existing file contents (if it exists)
        let contents = read_to_string(&file_path)?;

        // Check if the struct already exists
        let struct_exists = contents
            .lines()
            .any(|line| line.starts_with(&format!("pub struct {} ", self.struct_name)));

        if !struct_exists {
            // Append to the file
            let mut file = OpenOptions::new().append(true).open(&file_path)?;

            let contents = self.render()?;

            file.write_all(b"\n")?; // Add a newline if needed
            file.write_all(&contents.into_bytes())?;
            file.write_all(b"\n")?; // Add a newline if needed
            Ok(())
        } else {
            anyhow::bail!("Struct already exists")
        }
    }
}

impl Export for DbUpTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, "up", &self.render()?.into_bytes())
    }
}

impl Export for DbDownTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, "down", &self.render()?.into_bytes())
    }
}

fn create_migration_file(name: &str, ty: &str, content: &[u8]) -> Result<()> {
    let version = Utc::now().format("%Y-%m-%d-%H%M%S").to_string();
    let file_name = format!("migrations/{}_{}/{}.sql", version, name.to_lowercase(), ty);
    let file_path = Path::new(&file_name);

    if !file_path.exists() {
        create_dir_all(file_path.parent().unwrap())?;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    file.write_all(content)?;
    file.write_all(b"\n")?; // Add a newline if needed
    Ok(())
}

impl Export for PageTemplate<'_> {
    fn export(&self) -> Result<()> {
        let file_path = format!("src/admin/{}.rs", self.function_name);

        // Append to the file
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .open(file_path)?;

        let contents = self.render()?;

        file.write_all(&contents.into_bytes())?;
        Ok(())
    }
}
