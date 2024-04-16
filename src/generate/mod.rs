mod attribute;
mod crud;
mod data_types;
mod exporters;
mod options;
mod template;
mod transformers;

use self::attribute::Attribute;
use self::crud::CrudOperations;
use self::data_types::IDType;
use self::exporters::Export;
use self::options::GenerateOptions;
use self::template::{ApiTemplate, DbDownTemplate, DbUpTemplate, ModelTemplate, PageTemplate};
use self::transformers::{DataTypeTransformer, PostgresMigration, RustStruct};
use anyhow::{Context, Result};
use clap::Parser;
use console::Term;
use convert_case::{Case, Casing};
use dialoguer::{theme::ColorfulTheme, Input, Select};

trait FromClap: Sized {
    fn from_clap(str: &str) -> Result<Self>;
}

trait FromTerm<T>: Sized {
    fn from_term(term: &Term, theme: &ColorfulTheme) -> Result<T>;
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

    #[arg(short, long, value_delimiter = ',', value_parser = Attribute::from_clap, verbatim_doc_comment)]
    /// Attributes that the generated files should posses.
    /// These are constructed as {name}:{type}.
    /// Fox example: title:text.
    /// You can add an question mark at the end if the attribute is optional.
    pub attributes: Option<Vec<Attribute>>,

    #[arg(short = 'p', long, value_parser = CrudOperations::from_clap, verbatim_doc_comment)]
    /// Operations that should be generated for the api.
    /// Can be left out if no api is generated.
    /// Can be a comma separated list of the following:
    /// all, create, read, update, delete
    /// Or short notation without comma: crud
    pub operations: Option<CrudOperations>,
}

pub fn generate_files(args: GenerateArgs, term: Term, theme: ColorfulTheme) -> Result<()> {
    let selected_options = match args.options {
        Some(options) => options,
        None => GenerateOptions::from_term(&term, &theme)?,
    };

    let operations: Option<CrudOperations> = match args.operations {
        Some(operations) => Some(operations),
        None => {
            if selected_options
                .iter()
                .any(|x| *x == GenerateOptions::Routes)
            {
                Some(CrudOperations::from_term(&term, &theme)?)
            } else {
                None
            }
        }
    };

    let name: String = match args.name {
        Some(name) => name,
        None => Input::with_theme(&theme)
            .with_prompt("What is the name of the table/route?")
            .interact_on(&term)
            .unwrap(),
    };

    let id: IDType = match args.id {
        Some(id) => id,
        None => IDType::from_term(&term, &theme)?,
    };

    let attributes = match args.attributes {
        Some(attributes) => attributes,
        None => Attribute::from_term(&term, &theme)?,
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
                    crud_operations: operations.clone().context("No operations selected")?,
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
