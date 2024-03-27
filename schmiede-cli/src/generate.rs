use crate::data_types::{get_attributes, parse_cli_attributes, Attribute, DataType, IDType};
use crate::exporters::Export;
use crate::template::{DbDownTemplate, DbUpTemplate, ModelTemplate, PageTemplate};
use crate::transformers::{DataTypeTransformer, PostgresMigration, RustStruct};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use console::Term;
use convert_case::{Case, Casing};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::str::FromStr;
use strum::IntoEnumIterator;

const GENERATE_OPTIONS: [&str; 4] = [
    "SQL Migration",
    "Rust Struct",
    "API CRUD routes",
    "Admin Pages",
];

#[derive(ValueEnum, Clone, Debug)]
pub enum GenerateOptions {
    Sql,
    Struct,
    Routes,
    Admin,
}

impl GenerateOptions {
    fn from_usize(index: usize) -> Result<Self> {
        match index {
            0 => Ok(GenerateOptions::Sql),
            1 => Ok(GenerateOptions::Struct),
            2 => Ok(GenerateOptions::Routes),
            3 => Ok(GenerateOptions::Admin),
            _ => anyhow::bail!("Failed to convert option"),
        }
    }
}

fn attributes_parser(attribute: &str) -> Result<String> {
    let split_attribute = attribute.split(':');
    if split_attribute.clone().count() != 2 {
        anyhow::bail!(
            "Wrong format of attribute. {} should be in the format of name:type",
            attribute
        );
    }
    for (index, word) in split_attribute.enumerate() {
        let word = match word.ends_with('?') && index == 1 {
            true => &word[..word.len() - 1],
            false => &word,
        };
        for char in word.chars() {
            if !char.is_ascii_alphabetic() {
                anyhow::bail!("No number or special characters allowed: {}", char)
            }
        }
        if index == 1 {
            DataType::from_str(word).context(format!("{} is not a valid data type.", word))?;
        }
    }

    Ok(attribute.to_string())
}

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[clap(short, long)]
    /// Name of the generated files/data
    pub name: Option<String>,

    #[clap(short, long)]
    /// Type of id
    pub id: Option<IDType>,

    #[clap(short, long, value_delimiter = ',')]
    /// Options on what to generate
    pub options: Option<Vec<GenerateOptions>>,

    #[clap(short, long, value_delimiter = ',', value_parser = attributes_parser)]
    /// Attributes that the generated files should posses.
    /// These are constructed as {name}:{type}.
    /// Fox example: title:text.
    /// You can add an question mark at the end if the attribute is optional.
    pub attributes: Option<Vec<String>>,
}

pub fn generate_files(args: GenerateArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let selected_options = match args.options {
        Some(options) => options,
        None => {
            let options = MultiSelect::with_theme(&theme)
                .with_prompt("What do you want to create?")
                .items(&GENERATE_OPTIONS)
                .interact_on(&term)
                .unwrap();

            let mut res = vec![];
            for option in options {
                res.push(GenerateOptions::from_usize(option)?);
            }
            res
        }
    };

    if selected_options.is_empty() {
        term.write_line("No options selected, exiting...")?;
        return Ok(());
    }

    let name: String = match args.name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("What is the name of the table/route?")
            .interact_on(&term)
            .unwrap(),
    };

    let id: IDType = match args.id {
        Some(id) => id,
        None => {
            let id_options = IDType::iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            Select::with_theme(&theme)
                .with_prompt("Does the table/entry have an primary id?")
                .items(&id_options)
                .interact_on(&term)
                .unwrap()
                .into()
        }
    };

    let attributes = match args.attributes {
        Some(attributes) => parse_cli_attributes(attributes)?,
        None => get_attributes(&term, &theme)?,
    };

    for export_option in selected_options {
        match export_option {
            GenerateOptions::Sql => {
                let name = &name.to_case(Case::Snake);
                let up_template = DbUpTemplate {
                    name,
                    rows: get_rows(&attributes, export_option),
                    id: id.clone(),
                };
                let down_template = DbDownTemplate { name };
                up_template.export()?;
                down_template.export()?;
            }
            GenerateOptions::Struct => {
                let model_template = ModelTemplate {
                    id: id.clone(),
                    name: &name,
                    struct_name: &name.to_case(Case::Pascal),
                    rows: get_rows(&attributes, export_option),
                };
                model_template.export()?;
            }
            GenerateOptions::Routes => {}
            GenerateOptions::Admin => {
                let page_template = PageTemplate {
                    function_name: &name.to_case(Case::Snake),
                    model_name: &name.to_case(Case::Pascal),
                    route: &name.to_case(Case::Kebab),
                };

                page_template.export()?;
            }
        };
    }

    Ok(())
}

fn get_rows(data_types: &[Attribute], option: GenerateOptions) -> Vec<String> {
    let transformer: Box<dyn DataTypeTransformer> = match option {
        GenerateOptions::Sql => Box::new(PostgresMigration {}),
        GenerateOptions::Struct => Box::new(RustStruct {}),
        _ => return vec![],
    };

    data_types
        .iter()
        .map(|(name, data_type, optional)| {
            if *optional {
                transformer.get_optional_row(data_type, &name.to_case(Case::Snake))
            } else {
                transformer.get_row(data_type, &name.to_case(Case::Snake))
            }
        })
        .collect()
}
