use std::{fmt::Display, str::FromStr};

use anyhow::{Context, Result};
use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Clone)]
pub enum IDType {
    Uuid,
    Integer,
    None,
}

impl Display for IDType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IDType::Uuid => write!(fmt, "Uuid"),
            IDType::Integer => write!(fmt, "Integer"),
            IDType::None => write!(fmt, "None"),
        }
    }
}

impl FromStr for IDType {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self> {
        match input {
            "Uuid" => Ok(IDType::Uuid),
            "Integer" => Ok(IDType::Integer),
            "None" => Ok(IDType::None),
            _ => Err(anyhow::anyhow!("Invalid ID type")),
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
            DataType::Boolean => write!(fmt, "Boolean"),
            DataType::SmallInt => write!(fmt, "SmallInt"),
            DataType::Integer => write!(fmt, "Integer"),
            DataType::BigInt => write!(fmt, "BigInt"),
            DataType::Real => write!(fmt, "Real"),
            DataType::DoublePrecision => write!(fmt, "DoublePrecision"),
            DataType::Numeric(_, _) => write!(fmt, "Numeric"),
            DataType::Char(_) => write!(fmt, "Char"),
            DataType::VarChar(_) => write!(fmt, "VarChar",),
            DataType::Text => write!(fmt, "Text"),
            DataType::Bytea => write!(fmt, "Bytea"),
            DataType::Timestamp => write!(fmt, "Timestamp"),
            DataType::TimestampTZ => write!(fmt, "TimestampTZ"),
            DataType::Date => write!(fmt, "Date"),
            DataType::Time => write!(fmt, "Time"),
            DataType::TimeTZ => write!(fmt, "TimeTZ"),
            DataType::Interval => write!(fmt, "Interval"),
            DataType::Jsonb => write!(fmt, "Jsonb"),
            DataType::Uuid => write!(fmt, "Uuid"),
        }
    }
}

impl FromStr for DataType {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self> {
        match str {
            "Boolean" => Ok(DataType::Boolean),
            "SmallInt" => Ok(DataType::SmallInt),
            "Integer" => Ok(DataType::Integer),
            "BigInt" => Ok(DataType::BigInt),
            "Real" => Ok(DataType::Real),
            "DoublePrecision" => Ok(DataType::DoublePrecision),
            "Numeric" => Ok(DataType::Numeric(0, 0)),
            "Char" => Ok(DataType::Char(0)),
            "VarChar" => Ok(DataType::VarChar(0)),
            "Text" => Ok(DataType::Text),
            "Bytea" => Ok(DataType::Bytea),
            "Timestamp" => Ok(DataType::Timestamp),
            "TimestampTZ" => Ok(DataType::TimestampTZ),
            "Date" => Ok(DataType::Date),
            "Time" => Ok(DataType::Time),
            "TimeTZ" => Ok(DataType::TimeTZ),
            "Interval" => Ok(DataType::Interval),
            "Jsonb" => Ok(DataType::Jsonb),
            "Uuid" => Ok(DataType::Uuid),
            _ => Err(anyhow::anyhow!("Invalid data type")),
        }
    }
}

fn get_data_type(term: &Term, theme: &ColorfulTheme) -> Result<(String, DataType, bool)> {
    let data_types = DataType::iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();

    let name: String = Input::with_theme(theme)
        .with_prompt("Set your field name")
        .interact_on(term)
        .context("Failed to get field name")?;

    let data_type = FuzzySelect::with_theme(theme)
        .with_prompt("Pick a data type")
        .items(&data_types)
        .interact_on(term)
        .context("Failed to get data type")?;

    let optional = Confirm::with_theme(theme)
        .with_prompt("Is this field optional?")
        .interact_on(term)
        .context("Failed to get optional")?;

    let data_type = DataType::from_str(data_types.get(data_type).unwrap())?;

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

pub fn get_data_types(term: &Term, theme: &ColorfulTheme) -> Result<Vec<(String, DataType, bool)>> {
    let mut stop = false;
    let mut rows: Vec<(String, DataType, bool)> = vec![];

    while !stop {
        term.clear_screen().context("Failed to clear screen")?;
        if !rows.is_empty() {
            term.write_line("Current rows")
                .context("Failed to write line")?;
        }
        for row in rows.iter() {
            let mut res = format!("{}: {}", style(&row.0).cyan(), row.1);
            if row.2 {
                res.push_str(" (optional)");
            }
            term.write_line(&res).context("Failed to write line")?;
        }
        term.write_line("Create a new row")
            .context("Failed to write line")?;
        rows.push(get_data_type(term, theme)?);
        stop = !Confirm::with_theme(theme)
            .with_prompt("Do you want to create another row?")
            .interact_on(term)
            .context("Failed to get confirmation")?;
    }
    Ok(rows)
}
