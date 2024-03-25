use crate::data_types::{get_data_types, parse_cli_data_types, DataType, IDType};
use crate::exporters::Export;
use crate::template::{DbDownTemplate, DbUpTemplate, ModelTemplate, PageTemplate};
use crate::transformers::{DataTypeTransformer, PostgresMigration, RustStruct};
use anyhow::Result;
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
    SQL,
    Struct,
    Routes,
    Admin,
}

impl GenerateOptions {
    fn from_usize(index: usize) -> Result<Self> {
        match index {
            0 => Ok(GenerateOptions::SQL),
            1 => Ok(GenerateOptions::Struct),
            2 => Ok(GenerateOptions::Routes),
            3 => Ok(GenerateOptions::Admin),
            _ => anyhow::bail!("Failed to convert option"),
        }
    }
}

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[clap(short, long)]
    pub name: Option<String>,

    #[clap(short, long)]
    pub id: Option<IDType>,

    #[clap(short, long, value_delimiter = ',')]
    pub options: Option<Vec<GenerateOptions>>,

    #[clap(short, long, value_delimiter = ',', verbatim_doc_comment)]
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

    let id = match args.id {
        Some(id) => id,
        None => {
            let id_options = IDType::iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            let id = Select::with_theme(&theme)
                .with_prompt("Does the table/entry have an primary id?")
                .items(&id_options)
                .interact_on(&term)
                .unwrap();
            IDType::from_str(id_options.get(id).unwrap())?
        }
    };

    let data_types = match args.attributes {
        Some(attributes) => parse_cli_data_types(attributes)?,
        None => get_data_types(&term, &theme)?,
    };

    println!("{:?}", data_types);
    return Ok(());
    for export_option in selected_options {
        match export_option {
            GenerateOptions::SQL => {
                let name = &name.to_case(Case::Snake);
                let up_template = DbUpTemplate {
                    name,
                    rows: get_rows(&data_types, export_option),
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
                    rows: get_rows(&data_types, export_option),
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

fn get_rows(data_types: &Vec<(String, DataType, bool)>, option: GenerateOptions) -> Vec<String> {
    let transformer: Box<dyn DataTypeTransformer> = match option {
        GenerateOptions::SQL => Box::new(PostgresMigration {}),
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
