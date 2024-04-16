use crate::attribute::{get_term_attributes, Attribute};
use crate::crud::CrudOperations;
use crate::data_types::IDType;
use crate::exporters::Export;
use crate::template::{ApiTemplate, DbDownTemplate, DbUpTemplate, ModelTemplate, PageTemplate};
use crate::transformers::{DataTypeTransformer, PostgresMigration, RustStruct};
use anyhow::Result;
use clap::{Parser, ValueEnum};
use console::Term;
use convert_case::{Case, Casing};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
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

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[arg(short, long)]
    /// Name of the generated files/data
    pub name: Option<String>,

    #[arg(short, long)]
    /// Type of id
    pub id: Option<IDType>,

    #[arg(short, long, value_delimiter = ',')]
    /// Options on what to generate
    pub options: Option<Vec<GenerateOptions>>,

    #[arg(short, long, value_delimiter = ',', value_parser = Attribute::from_str, verbatim_doc_comment)]
    /// Attributes that the generated files should posses.
    /// These are constructed as {name}:{type}.
    /// Fox example: title:text.
    /// You can add an question mark at the end if the attribute is optional.
    pub attributes: Option<Vec<Attribute>>,

    #[arg(short = 'p', long, value_parser = CrudOperations::from_cli, verbatim_doc_comment)]
    /// Operations that should be generated for the api.
    /// Can be left out if no api is generated.
    /// Can be a comma separated list of the following:
    /// all, create, read, update, delete
    /// Or short notation without comma: crud
    pub operations: Option<CrudOperations>,
}

pub fn generate_files(args: GenerateArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    println!("{:?}", args);
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
        Some(attributes) => attributes,
        None => get_term_attributes(&term, &theme)?,
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
            GenerateOptions::Routes => {
                let api_template = ApiTemplate {
                    name: &name,
                    struct_name: &name.to_case(Case::Pascal),
                    crud_operations: CrudOperations::All,
                };
                api_template.export()?;
            }
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

fn get_rows(attributes: &[Attribute], option: GenerateOptions) -> Vec<String> {
    let transformer: Box<dyn DataTypeTransformer> = match option {
        GenerateOptions::Sql => Box::new(PostgresMigration {}),
        GenerateOptions::Struct => Box::new(RustStruct {}),
        _ => return vec![],
    };

    attributes
        .iter()
        .map(|attribute| {
            if attribute.optional {
                transformer.get_optional_row(&attribute.data_type, &attribute.name)
            } else {
                transformer.get_row(&attribute.data_type, &attribute.name)
            }
        })
        .collect()
}
