use super::FromTerm;
use anyhow::{Context, Result};
use clap::ValueEnum;
use console::Term;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum GenerateOptions {
    Sql,
    Struct,
    Routes,
    //Admin,
}

impl GenerateOptions {
    const VALUES: [&'static str; 3] = ["sql", "struct", "routes"];

    pub fn requires_id(&self) -> bool {
        matches!(self, GenerateOptions::Struct | GenerateOptions::Sql)
    }

    pub fn requires_attributes(&self) -> bool {
        matches!(self, GenerateOptions::Struct | GenerateOptions::Sql)
    }

    pub fn requires_operations(&self) -> bool {
        matches!(self, GenerateOptions::Routes)
    }
}

impl TryFrom<usize> for GenerateOptions {
    type Error = anyhow::Error;
    fn try_from(index: usize) -> Result<Self> {
        match index {
            0 => Ok(GenerateOptions::Sql),
            1 => Ok(GenerateOptions::Struct),
            2 => Ok(GenerateOptions::Routes),
            //3 => Ok(GenerateOptions::Admin),
            _ => anyhow::bail!("Failed to convert option"),
        }
    }
}

impl FromTerm<Vec<Self>> for GenerateOptions {
    fn from_term(term: &Term, theme: &ColorfulTheme) -> Result<Vec<Self>> {
        let options = MultiSelect::with_theme(theme)
            .with_prompt("Select what to generate")
            .items(&GenerateOptions::VALUES)
            .interact_on(term)
            .context("Failed to get generate option")?;

        let mut res = vec![];
        for option in options {
            res.push(option.try_into()?);
        }
        if res.is_empty() {
            anyhow::bail!("No options selected...");
        }
        Ok(res)
    }
}
