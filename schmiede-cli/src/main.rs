mod data_types;
mod exporters;
mod init;
mod template;
mod transformers;

use std::str::FromStr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::Term;
use convert_case::{Case, Casing};
use data_types::{get_data_types, IDType};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use exporters::Export;
use strum::IntoEnumIterator;
use template::{DbDownTemplate, DbUpTemplate};
use transformers::{DataTypeTransformer, PostgresMigration, RustStruct};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Gen,
    Init,
}

const EXPORT_OPTIONS: [&str; 4] = ["SQL Migration", "Rust Structs", "API routes", "Admin Pages"];

fn main() -> Result<()> {
    let args = Args::parse();
    let mut generate = false;
    match args.cmd {
        Some(Commands::Gen) => generate = true,
        Some(Commands::Init) => println!("Initializing..."),
        None => {}
    };
    if generate {
        init::initialize_starter("test")?;
        return Ok(());
    }
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    let name: String = Input::with_theme(&theme)
        .with_prompt("What is the name of the table/entry?")
        .interact_on(&term)
        .unwrap();

    let selected_exports = MultiSelect::with_theme(&theme)
        .with_prompt("What do you want to create?")
        .items(&EXPORT_OPTIONS)
        .interact_on(&term)
        .unwrap();

    if selected_exports.is_empty() {
        term.write_line("No exports selected, exiting...")?;
        return Ok(());
    }
    let id_options = IDType::iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    let id = Select::with_theme(&theme)
        .with_prompt("Does the table/entry have an primary id?")
        .items(&id_options)
        .interact_on(&term)
        .unwrap();

    let id = IDType::from_str(id_options.get(id).unwrap())?;

    let data_types = get_data_types(&term, &theme)?;

    for export_option in selected_exports {
        let transformer: Box<dyn DataTypeTransformer> = match export_option {
            0 => Box::new(PostgresMigration {}),
            _ => Box::new(RustStruct {}),
        };

        let rows: Vec<String> = data_types
            .iter()
            .map(|(name, data_type, optional)| {
                if *optional {
                    transformer.get_optional_row(data_type, &name.to_case(Case::Snake))
                } else {
                    transformer.get_row(data_type, &name.to_case(Case::Snake))
                }
            })
            .collect();

        match export_option {
            0 => {
                let name = &name.to_case(Case::Snake);
                let up_template = DbUpTemplate {
                    name,
                    rows: rows.clone(),
                    id: id.clone(),
                };
                let down_template = DbDownTemplate { name };
                if generate {
                    up_template.export()?;
                    down_template.export()?;
                }
            }
            1 => {
                let model_template = template::ModelTemplate {
                    id: id.clone(),
                    name: &name,
                    struct_name: &name.to_case(Case::Pascal),
                    rows: rows.clone(),
                };
                if generate {
                    model_template.export()?;
                }
            }
            3 => {
                let page_template = template::PageTemplate {
                    function_name: &name.to_case(Case::Snake),
                    model_name: &name.to_case(Case::Pascal),
                    route: &name.to_case(Case::Kebab),
                };

                if generate {
                    page_template.export()?;
                }
            }
            _ => {}
        };
    }

    Ok(())
}
