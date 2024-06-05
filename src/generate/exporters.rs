use super::template::{
    AxumDieselTemplate, AxumSqlxTemplate, DieselDownTemplate, DieselModelTemplate,
    DieselUpTemplate, SqlxDownTemplate, SqlxModelTemplate, SqlxUpTemplate,
};
use anyhow::{Context, Result};
use askama::Template;
use chrono::Utc;
use convert_case::{Case, Casing};
use std::fs::{create_dir_all, read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;

pub trait Export {
    fn export(&self) -> Result<()>;
}

impl Export for SqlxModelTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_model_file(self.struct_name, &self.render()?.into_bytes())
    }
}

impl Export for DieselModelTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_model_file(self.struct_name, &self.render()?.into_bytes())
    }
}

fn create_model_file(name: &str, content: &[u8]) -> Result<()> {
    let file_path = String::from("src/common/models.rs");

    let exists = Path::new(&file_path).exists();

    if !exists {
        create_dir_all(Path::new(&file_path).parent().unwrap())?;
    }

    // Check if the struct already exists
    let struct_exists = match exists {
        true => {
            // Read existing file contents (if it exists)
            let contents = read_to_string(&file_path)?;
            contents
                .lines()
                .any(|line| line.starts_with(&format!("pub struct {} ", name)))
        }
        false => false,
    };

    if !struct_exists {
        // Append to the file
        let mut file = OpenOptions::new()
            .append(true)
            .create(!exists)
            .open(&file_path)
            .context(format!("Failed to create model for {}", name))?;

        file.write_all(b"\n")?; // Add a newline if needed
        file.write_all(content)?;
        file.write_all(b"\n")?; // Add a newline if needed
        Ok(())
    } else {
        anyhow::bail!("Struct already exists")
    }
}

impl Export for SqlxUpTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, false, "up", &self.render()?.into_bytes())
    }
}

impl Export for SqlxDownTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, false, "down", &self.render()?.into_bytes())
    }
}

impl Export for DieselUpTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, true, "up", &self.render()?.into_bytes())
    }
}

impl Export for DieselDownTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        create_migration_file(self.name, true, "down", &self.render()?.into_bytes())
    }
}

fn create_migration_file(name: &str, has_dir: bool, ty: &str, content: &[u8]) -> Result<()> {
    let timestamp_format = match has_dir {
        true => "%Y-%m-%d-%H%M%S",
        false => "%Y%m%d%H%M%S",
    };
    let timestamp = Utc::now().format(timestamp_format).to_string();

    let file_name = match has_dir {
        true => format!(
            "migrations/{}_{}/{}.sql",
            timestamp,
            name.to_lowercase(),
            ty
        ),
        false => format!(
            "migrations/{}_{}.{}.sql",
            timestamp,
            name.to_lowercase(),
            ty
        ),
    };
    let file_path = Path::new(&file_name);

    if !file_path.exists() {
        create_dir_all(file_path.parent().unwrap())?;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .context(format!("Failed to create migration for {}", name))?;

    file.write_all(content)?;
    file.write_all(b"\n")?; // Add a newline if needed
    Ok(())
}

impl Export for AxumDieselTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        let file_path = format!("src/routes/{}.rs", self.name.to_case(Case::Snake));

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .context(format!("Failed to create template for {}", self.name))?;

        let contents = self.render()?;

        file.write_all(&contents.into_bytes())?;
        Ok(())
    }
}

impl Export for AxumSqlxTemplate<'_>
where
    Self: Template,
{
    fn export(&self) -> Result<()> {
        let file_path = format!("src/routes/{}.rs", self.name.to_case(Case::Snake));

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .context(format!("Failed to create template for {}", self.name))?;

        let contents = self.render()?;

        file.write_all(&contents.into_bytes())?;
        Ok(())
    }
}

/*
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
*/
