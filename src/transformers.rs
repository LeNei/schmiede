use crate::data_types::DataType;
use convert_case::{Case, Casing};

pub trait DataTypeTransformer {
    fn get_type(&self, row_type: &DataType) -> String;
    fn get_row(&self, row_type: &DataType, name: &str) -> String;
    fn get_optional_row(&self, row_type: &DataType, name: &str) -> String;
}

pub struct RustStruct {}

impl DataTypeTransformer for RustStruct {
    fn get_type(&self, row_type: &DataType) -> String {
        match row_type {
            DataType::Boolean => "bool".to_string(),
            DataType::SmallInt => "i16".to_string(),
            DataType::Integer => "i32".to_string(),
            DataType::BigInt => "i64".to_string(),
            DataType::Real => "f32".to_string(),
            DataType::DoublePrecision => "f64".to_string(),
            DataType::Numeric(_, _) => "f64".to_string(),
            DataType::Char(_) => "char".to_string(),
            DataType::VarChar(_) => "String".to_string(),
            DataType::Text => "String".to_string(),
            DataType::Bytea => "Vec<u8>".to_string(),
            DataType::Timestamp => "chrono::NaiveDateTime".to_string(),
            DataType::TimestampTZ => "chrono::DateTime<chrono::Utc>".to_string(),
            DataType::Date => "chrono::NaiveDate".to_string(),
            DataType::Time => "chrono::NaiveTime".to_string(),
            DataType::TimeTZ => "chrono::NaiveTime".to_string(),
            DataType::Interval => "chrono::Duration".to_string(),
            DataType::Jsonb => "serde_json::Value".to_string(),
            DataType::Uuid => "uuid::Uuid".to_string(),
        }
    }

    fn get_row(&self, row_type: &DataType, name: &str) -> String {
        format!("{}: {}", name.to_case(Case::Snake), self.get_type(row_type))
    }

    fn get_optional_row(&self, row_type: &DataType, name: &str) -> String {
        format!(
            "{}: Option<{}>",
            name.to_case(Case::Snake),
            self.get_type(row_type)
        )
    }
}

pub struct PostgresMigration {}

impl DataTypeTransformer for PostgresMigration {
    fn get_type(&self, row_type: &DataType) -> String {
        match row_type {
            DataType::Boolean => "BOOLEAN".to_string(),
            DataType::SmallInt => "SMALLINT".to_string(),
            DataType::Integer => "INTEGER".to_string(),
            DataType::BigInt => "BIGINT".to_string(),
            DataType::Real => "REAL".to_string(),
            DataType::DoublePrecision => "DOUBLE PRECISION".to_string(),
            DataType::Numeric(precision, scale) => {
                format!("NUMERIC({}, {})", precision, scale)
            }
            DataType::Char(length) => format!("CHAR({})", length),
            DataType::VarChar(length) => format!("VARCHAR({})", length),
            DataType::Text => "TEXT".to_string(),
            DataType::Bytea => "BYTEA".to_string(),
            DataType::Timestamp => "TIMESTAMP".to_string(),
            DataType::TimestampTZ => "TIMESTAMPTZ".to_string(),
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::TimeTZ => "TIMETZ".to_string(),
            DataType::Interval => "INTERVAL".to_string(),
            DataType::Jsonb => "JSONB".to_string(),
            DataType::Uuid => "UUID".to_string(),
        }
    }

    fn get_row(&self, row_type: &DataType, name: &str) -> String {
        format!(
            "{} {} NOT NULL",
            name.to_case(Case::Snake),
            self.get_type(row_type)
        )
    }

    fn get_optional_row(&self, row_type: &DataType, name: &str) -> String {
        format!("{} {}", name.to_case(Case::Snake), self.get_type(row_type))
    }
}
