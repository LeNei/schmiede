use anyhow::{Context, Result};
use clap::ValueEnum;
use console::{style, Term};
use convert_case::{Case, Casing};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use std::{fmt::Display, str::FromStr};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone, ValueEnum)]
pub enum IDType {
    Uuid,
    Int,
    None,
}

impl Display for IDType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IDType::Uuid => write!(fmt, "uuid"),
            IDType::Int => write!(fmt, "int"),
            IDType::None => write!(fmt, "none"),
        }
    }
}

impl From<usize> for IDType {
    fn from(value: usize) -> Self {
        match value {
            0 => IDType::Uuid,
            1 => IDType::Int,
            2 => IDType::None,
            _ => panic!("Invalid enum"),
        }
    }
}

#[derive(Debug, EnumIter)]
pub enum DataType {
    // Common Types
    Boolean,
    SmallInt,
    Integer,
    BigInt,
    Real,
    DoublePrecision,
    Numeric(u32, u32), // Numeric(precision, scale)
    Char(u32),         // Char(length)
    VarChar(u32),      // VarChar(length)
    Text,
    Bytea,
    Timestamp,
    TimestampTZ, // Timestamp with time zone
    Date,
    Time,
    TimeTZ, // Time with time zone
    Interval,

    // More specialized
    Jsonb,
    Uuid,
}

impl Display for DataType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataType::Boolean => write!(fmt, "bool"),
            DataType::SmallInt => write!(fmt, "smallInt"),
            DataType::Integer => write!(fmt, "int"),
            DataType::BigInt => write!(fmt, "bigInt"),
            DataType::Real => write!(fmt, "real"),
            DataType::DoublePrecision => write!(fmt, "doublePrecision"),
            DataType::Numeric(_, _) => write!(fmt, "numeric"),
            DataType::Char(_) => write!(fmt, "char"),
            DataType::VarChar(_) => write!(fmt, "varChar",),
            DataType::Text => write!(fmt, "text"),
            DataType::Bytea => write!(fmt, "bytea"),
            DataType::Timestamp => write!(fmt, "timestamp"),
            DataType::TimestampTZ => write!(fmt, "timestampTZ"),
            DataType::Date => write!(fmt, "date"),
            DataType::Time => write!(fmt, "time"),
            DataType::TimeTZ => write!(fmt, "timeTZ"),
            DataType::Interval => write!(fmt, "interval"),
            DataType::Jsonb => write!(fmt, "jsonb"),
            DataType::Uuid => write!(fmt, "uuid"),
        }
    }
}

impl FromStr for DataType {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self> {
        match str.to_case(Case::Camel).as_str() {
            "bool" => Ok(DataType::Boolean),
            "smallInt" => Ok(DataType::SmallInt),
            "int" => Ok(DataType::Integer),
            "bigInt" => Ok(DataType::BigInt),
            "real" => Ok(DataType::Real),
            "doublePrecision" => Ok(DataType::DoublePrecision),
            "numeric" => Ok(DataType::Numeric(0, 0)),
            "char" => Ok(DataType::Char(0)),
            "varChar" => Ok(DataType::VarChar(0)),
            "text" => Ok(DataType::Text),
            "bytea" => Ok(DataType::Bytea),
            "timestamp" => Ok(DataType::Timestamp),
            "timestampTZ" => Ok(DataType::TimestampTZ),
            "date" => Ok(DataType::Date),
            "time" => Ok(DataType::Time),
            "timeTZ" => Ok(DataType::TimeTZ),
            "interval" => Ok(DataType::Interval),
            "jsonb" => Ok(DataType::Jsonb),
            "uuid" => Ok(DataType::Uuid),
            _ => Err(anyhow::anyhow!("Invalid data type")),
        }
    }
}

impl From<usize> for DataType {
    fn from(value: usize) -> Self {
        match value {
            0 => DataType::Boolean,
            1 => DataType::SmallInt,
            2 => DataType::Integer,
            3 => DataType::BigInt,
            4 => DataType::Real,
            5 => DataType::DoublePrecision,
            6 => DataType::Numeric(0, 0),
            7 => DataType::Char(0),
            8 => DataType::VarChar(0),
            9 => DataType::Text,
            10 => DataType::Bytea,
            11 => DataType::Timestamp,
            12 => DataType::TimestampTZ,
            13 => DataType::Date,
            14 => DataType::Time,
            15 => DataType::TimeTZ,
            16 => DataType::Interval,
            17 => DataType::Jsonb,
            18 => DataType::Uuid,
            _ => panic!("False id"),
        }
    }
}

pub type Attribute = (String, DataType, bool);

pub fn get_attributes(term: &Term, theme: &ColorfulTheme) -> Result<Vec<Attribute>> {
    let mut stop = false;
    let mut attributes: Vec<Attribute> = vec![];

    while !stop {
        term.clear_screen().context("Failed to clear screen")?;
        if !attributes.is_empty() {
            term.write_line("Current rows")
                .context("Failed to write line")?;
        }
        for row in attributes.iter() {
            let mut res = format!("{}: {}", style(&row.0).cyan(), row.1);
            if row.2 {
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
    Ok((name, data_type, optional))
}

pub fn parse_cli_attributes(data_types: Vec<String>) -> Result<Vec<Attribute>> {
    let mut res = vec![];
    for ty in data_types {
        res.push(parse_cli_attribute(&ty)?);
    }
    Ok(res)
}

fn parse_cli_attribute(data_type: &str) -> Result<Attribute> {
    let (name, mut data_type_definition) = data_type.split_once(':').unwrap();
    let mut required = true;
    if data_type_definition.ends_with('?') {
        required = false;
        data_type_definition = &data_type_definition[..data_type_definition.len() - 1];
    }
    let data_type = DataType::from_str(&data_type_definition)?;

    Ok((name.to_string(), data_type, required))
}
