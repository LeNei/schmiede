use super::FromTerm;
use anyhow::{Context, Result};
use clap::ValueEnum;
use console::Term;
use convert_case::{Case, Casing};
use dialoguer::{theme::ColorfulTheme, Select};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, ValueEnum)]
pub enum IDType {
    Uuid,
    Int,
    None,
}

impl IDType {
    pub fn values() -> Vec<&'static str> {
        vec!["uuid", "int", "none"]
    }
}

impl FromTerm<Self> for IDType {
    fn from_term(term: &Term, theme: &ColorfulTheme) -> Result<Self> {
        Select::with_theme(theme)
            .with_prompt("Does the table/entry have an primary id?")
            .items(&IDType::values())
            .interact_on(term)
            .context("Failed to get id type")?
            .try_into()
    }
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

impl TryFrom<usize> for IDType {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            0 => Ok(IDType::Uuid),
            1 => Ok(IDType::Int),
            2 => Ok(IDType::None),
            _ => anyhow::bail!("Failed to convert IDType from usize"),
        }
    }
}

#[derive(Debug, Clone)]
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

impl DataType {
    pub fn values() -> Vec<&'static str> {
        vec![
            "bool",
            "smallInt",
            "int",
            "bigInt",
            "real",
            "doublePrecision",
            "numeric",
            "char",
            "varChar",
            "text",
            "bytea",
            "timestamp",
            "timestampTZ",
            "date",
            "time",
            "timeTZ",
            "interval",
            "jsonb",
            "uuid",
        ]
    }
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

impl TryFrom<usize> for DataType {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            0 => Ok(DataType::Boolean),
            1 => Ok(DataType::SmallInt),
            2 => Ok(DataType::Integer),
            3 => Ok(DataType::BigInt),
            4 => Ok(DataType::Real),
            5 => Ok(DataType::DoublePrecision),
            6 => Ok(DataType::Numeric(0, 0)),
            7 => Ok(DataType::Char(0)),
            8 => Ok(DataType::VarChar(0)),
            9 => Ok(DataType::Text),
            10 => Ok(DataType::Bytea),
            11 => Ok(DataType::Timestamp),
            12 => Ok(DataType::TimestampTZ),
            13 => Ok(DataType::Date),
            14 => Ok(DataType::Time),
            15 => Ok(DataType::TimeTZ),
            16 => Ok(DataType::Interval),
            17 => Ok(DataType::Jsonb),
            18 => Ok(DataType::Uuid),
            _ => anyhow::bail!("Failed to convert data type from usize"),
        }
    }
}
