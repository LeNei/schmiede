use super::{FromClap, FromTerm};
use anyhow::{Context, Result};
use console::Term;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum CrudOperations {
    All,
    Specific(Vec<SpecificOperation>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpecificOperation {
    Create,
    Read,
    Update,
    Delete,
}

impl FromClap for CrudOperations {
    fn from_clap(operations: &str) -> Result<Self> {
        if Self::from_str(operations).is_ok() {
            return Ok(Self::All);
        }
        let mut res: Vec<SpecificOperation> = vec![];
        if operations.contains(',') {
            for operation in operations.split(',') {
                res.push(operation.parse()?);
            }
        } else if let Ok(operation) = operations.parse() {
            res.push(operation);
        } else {
            for operation in operations.chars() {
                res.push(operation.to_string().parse()?);
            }
        }

        if res.is_empty() {
            anyhow::bail!("Failed to convert crud operations");
        }

        if SpecificOperation::check_duplicate_values(&res) {
            anyhow::bail!(
                "Failed to convert crud operations. It seems like you have duplicate operations."
            );
        }

        if res.len() == 4 {
            Ok(Self::All)
        } else {
            Ok(Self::Specific(res))
        }
    }
}

impl FromTerm<Self> for CrudOperations {
    fn from_term(term: &Term, theme: &ColorfulTheme) -> Result<Self> {
        let operations = MultiSelect::with_theme(theme)
            .with_prompt("What operations should the api have?")
            .items(&SpecificOperation::values())
            .interact_on(term)
            .context("Failed to get operations")?;

        if operations.is_empty() {
            anyhow::bail!("No operations selected");
        }

        if operations.len() == 4 {
            return Ok(Self::All);
        }

        let mut res = vec![];

        for operation in operations {
            res.push(operation.try_into()?);
        }

        Ok(Self::Specific(res))
    }
}

impl FromStr for CrudOperations {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self> {
        match str {
            "all" => Ok(CrudOperations::All),
            "a" => Ok(CrudOperations::All),
            _ => anyhow::bail!("Failed to convert crud operations"),
        }
    }
}

impl SpecificOperation {
    fn values() -> Vec<&'static str> {
        vec!["create", "read", "update", "delete"]
    }

    fn check_duplicate_values(values: &[SpecificOperation]) -> bool {
        let mut found = vec![];
        for operation in values {
            if found.iter().any(|o| *o == operation) {
                return true;
            }
            found.push(operation);
        }
        false
    }
}

impl TryFrom<usize> for SpecificOperation {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            0 => Ok(SpecificOperation::Create),
            1 => Ok(SpecificOperation::Read),
            2 => Ok(SpecificOperation::Update),
            3 => Ok(SpecificOperation::Delete),
            _ => anyhow::bail!("Failed to convert operation"),
        }
    }
}

impl FromStr for SpecificOperation {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self> {
        match str {
            "create" => Ok(SpecificOperation::Create),
            "c" => Ok(SpecificOperation::Create),
            "read" => Ok(SpecificOperation::Read),
            "r" => Ok(SpecificOperation::Read),
            "update" => Ok(SpecificOperation::Update),
            "u" => Ok(SpecificOperation::Update),
            "delete" => Ok(SpecificOperation::Delete),
            "d" => Ok(SpecificOperation::Delete),
            _ => anyhow::bail!("Invalid operation"),
        }
    }
}
