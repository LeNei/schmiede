use crate::data_types::DataType;
use anyhow::{Context, Result};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use std::str::FromStr;
use strum::IntoEnumIterator;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub data_type: DataType,
    pub optional: bool,
}

impl Attribute {
    pub fn from_str(attribute: &str) -> Result<Self> {
        let (name, mut data_type_definition) = attribute.split_once(':').unwrap();
        let mut required = true;
        if data_type_definition.ends_with('?') {
            required = false;
            data_type_definition = &data_type_definition[..data_type_definition.len() - 1];
        }
        let data_type = DataType::from_str(data_type_definition)?;

        Ok(Attribute {
            name: name.to_string(),
            data_type,
            optional: !required,
        })
    }
}

pub fn get_term_attributes(term: &Term, theme: &ColorfulTheme) -> Result<Vec<Attribute>> {
    let mut stop = false;
    let mut attributes: Vec<Attribute> = vec![];

    while !stop {
        term.clear_screen().context("Failed to clear screen")?;
        if !attributes.is_empty() {
            term.write_line("Current rows")
                .context("Failed to write line")?;
        }

        for attribute in attributes.iter() {
            let mut res = format!("{}: {}", style(&attribute.name).cyan(), attribute.data_type);
            if attribute.optional {
                res.push_str(" (optional)");
            }
            term.write_line(&res).context("Failed to write line")?;
        }

        term.write_line("Create a new row")
            .context("Failed to write line")?;
        attributes.push(get_attribute(term, theme)?);
        stop = !Confirm::with_theme(theme)
            .with_prompt("Do you want to create another row?")
            .interact_on(term)
            .context("Failed to get confirmation")?;
    }
    Ok(attributes)
}

fn get_attribute(term: &Term, theme: &ColorfulTheme) -> Result<Attribute> {
    let data_types = DataType::iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();

    let name: String = Input::with_theme(theme)
        .with_prompt("Set your field name")
        .interact_on(term)
        .context("Failed to get field name")?;

    let data_type: DataType = FuzzySelect::with_theme(theme)
        .with_prompt("Pick a data type")
        .items(&data_types)
        .interact_on(term)
        .context("Failed to get data type")?
        .into();

    let optional = Confirm::with_theme(theme)
        .with_prompt("Is this field optional?")
        .interact_on(term)
        .context("Failed to get optional")?;

    let data_type = match data_type {
        DataType::Numeric(_, _) => {
            let precision: u32 = Input::with_theme(theme)
                .with_prompt("Set precision")
                .interact_on(term)
                .context("Failed to get precision")?;
            let scale: u32 = Input::with_theme(theme)
                .with_prompt("Set scale")
                .interact_on(term)
                .context("Failed to get scale")?;
            DataType::Numeric(precision, scale)
        }
        DataType::Char(_) | DataType::VarChar(_) => {
            let length: u32 = Input::with_theme(theme)
                .with_prompt("Set length")
                .interact_on(term)
                .context("Failed to get length")?;

            match data_type {
                DataType::VarChar(_) => DataType::VarChar(length),
                DataType::Char(_) => DataType::Char(length),
                _ => data_type,
            }
        }
        _ => data_type,
    };
    Ok(Attribute {
        name,
        data_type,
        optional,
    })
}
