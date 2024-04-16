use anyhow::{Context, Result};
use console::Term;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

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

const OPERATIONS: [&str; 4] = ["create", "read", "update", "delete"];

impl CrudOperations {
    pub fn from_clap(operations: &str) -> Result<Self> {
        if Self::from_str(operations).is_ok() {
            return Ok(Self::All);
        }
        let mut res = vec![];
        if operations.contains(',') {
            for operation in operations.split(',') {
                res.push(SpecificOperation::from_str(operation)?);
            }
        } else if let Ok(operation) = SpecificOperation::from_str(operations) {
            res.push(operation);
        } else {
            for operation in operations.chars() {
                res.push(SpecificOperation::from_str(&operation.to_string())?);
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

    fn from_str(operation: &str) -> Result<Self> {
        match operation {
            "all" => Ok(CrudOperations::All),
            "a" => Ok(CrudOperations::All),
            _ => anyhow::bail!("Failed to convert crud operations"),
        }
    }

    pub fn from_cli(term: &Term, theme: &ColorfulTheme) -> Result<Self> {
        let operations = MultiSelect::with_theme(theme)
            .with_prompt("What operations should the api have?")
            .items(&OPERATIONS)
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
            res.push(SpecificOperation::from_str(OPERATIONS[operation])?);
        }

        Ok(Self::Specific(res))
    }
}

impl SpecificOperation {
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

    fn from_str(operation: &str) -> Result<Self> {
        match operation {
            "create" => Ok(SpecificOperation::Create),
            "c" => Ok(SpecificOperation::Create),
            "read" => Ok(SpecificOperation::Read),
            "r" => Ok(SpecificOperation::Read),
            "update" => Ok(SpecificOperation::Update),
            "u" => Ok(SpecificOperation::Update),
            "delete" => Ok(SpecificOperation::Delete),
            "d" => Ok(SpecificOperation::Delete),
            _ => anyhow::bail!("Failed to convert specific operation"),
        }
    }
}
