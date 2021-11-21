mod conversion;
mod custom_value;

use std::{cmp::Ordering, fmt::Display, hash::Hasher};

use conversion::{Column, ColumnMap};
use indexmap::map::IndexMap;
use nu_protocol::{did_you_mean, ShellError, Span, Value};
use polars::prelude::{DataFrame, PolarsObject, Series};
use serde::{Deserialize, Serialize};

// DataFrameValue is an encapsulation of Nushell Value that can be used
// to define the PolarsObject Trait. The polars object trait allows to
// create dataframes with mixed datatypes
#[derive(Clone, Debug)]
pub struct DataFrameValue(Value);

impl DataFrameValue {
    fn new(value: Value) -> Self {
        Self(value)
    }

    fn get_value(&self) -> Value {
        self.0.clone()
    }
}

impl Display for DataFrameValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get_type())
    }
}

impl Default for DataFrameValue {
    fn default() -> Self {
        Self(Value::Nothing {
            span: Span::unknown(),
        })
    }
}

impl PartialEq for DataFrameValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.partial_cmp(&other.0).map_or(false, Ordering::is_eq)
    }
}
impl Eq for DataFrameValue {}

impl std::hash::Hash for DataFrameValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.0 {
            Value::Nothing { .. } => 0.hash(state),
            Value::Int { val, .. } => val.hash(state),
            Value::String { val, .. } => val.hash(state),
            // TODO. Define hash for the rest of types
            _ => {}
        }
    }
}

impl PolarsObject for DataFrameValue {
    fn type_name() -> &'static str {
        "value"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NuDataFrame(DataFrame);

impl NuDataFrame {
    pub fn new(dataframe: DataFrame) -> Self {
        Self(dataframe)
    }

    pub fn dataframe_into_value(dataframe: DataFrame, span: Span) -> Value {
        Value::CustomValue {
            val: Box::new(Self::new(dataframe)),
            span,
        }
    }

    pub fn to_value(self, span: Span) -> Value {
        Value::CustomValue {
            val: Box::new(self),
            span,
        }
    }

    pub fn try_from_iter<T>(iter: T) -> Result<Self, ShellError>
    where
        T: Iterator<Item = Value>,
    {
        // Dictionary to store the columnar data extracted from
        // the input. During the iteration we check if the values
        // have different type
        let mut column_values: ColumnMap = IndexMap::new();

        for value in iter {
            match value {
                Value::List { vals, .. } => {
                    let cols = (0..vals.len())
                        .map(|i| format!("{}", i))
                        .collect::<Vec<String>>();

                    conversion::insert_record(&mut column_values, &cols, &vals)?
                }
                Value::Record { cols, vals, .. } => {
                    conversion::insert_record(&mut column_values, &cols, &vals)?
                }
                _ => {
                    let key = "0".to_string();
                    conversion::insert_value(value, key, &mut column_values)?
                }
            }
        }

        conversion::from_parsed_columns(column_values)
    }

    pub fn try_from_series(columns: Vec<Series>) -> Result<Self, ShellError> {
        let dataframe = DataFrame::new(columns)
            .map_err(|e| ShellError::InternalError(format!("Unable to create DataFrame: {}", e)))?;

        Ok(Self::new(dataframe))
    }

    pub fn try_from_columns(columns: Vec<Column>) -> Result<Self, ShellError> {
        let mut column_values: ColumnMap = IndexMap::new();

        for column in columns {
            let name = column.name().to_string();
            for value in column {
                conversion::insert_value(value, name.clone(), &mut column_values)?;
            }
        }

        conversion::from_parsed_columns(column_values)
    }

    pub fn column(&self, column: &str, span: Span) -> Result<Self, ShellError> {
        let s = self.0.column(column).map_err(|_| {
            let possibilities = self
                .0
                .get_column_names()
                .iter()
                .map(|name| name.to_string())
                .collect::<Vec<String>>();

            let option = did_you_mean(&possibilities, column).unwrap_or_else(|| column.to_string());
            ShellError::DidYouMean(option, span)
        })?;

        let dataframe = DataFrame::new(vec![s.clone()])
            .map_err(|e| ShellError::InternalError(e.to_string()))?;

        Ok(Self(dataframe))
    }

    pub fn is_series(&self) -> bool {
        self.0.width() == 1
    }

    pub fn as_series(&self, _span: Span) -> Result<Series, ShellError> {
        if !self.is_series() {
            return Err(ShellError::InternalError(
                "DataFrame cannot be used as Series".into(),
            ));
        }

        let series = self
            .0
            .get_columns()
            .get(0)
            .expect("We have already checked that the width is 1");

        Ok(series.clone())
    }

    pub fn get_value(&self, row: usize, span: Span) -> Result<Value, ShellError> {
        let series = self.as_series(Span::unknown())?;
        let column = conversion::create_column(&series, row, row + 1)?;

        if column.len() == 0 {
            Err(ShellError::AccessBeyondEnd(series.len(), span))
        } else {
            let value = column
                .into_iter()
                .next()
                .expect("already checked there is a value");
            Ok(value)
        }
    }

    // Print is made out a head and if the dataframe is too large, then a tail
    pub fn print(&self) -> Result<Vec<Value>, ShellError> {
        let df = &self.0;
        let size: usize = 20;

        if df.height() > size {
            let sample_size = size / 2;
            let mut values = self.head(Some(sample_size))?;
            conversion::add_separator(&mut values, df);
            let remaining = df.height() - sample_size;
            let tail_size = remaining.min(sample_size);
            let mut tail_values = self.tail(Some(tail_size))?;
            values.append(&mut tail_values);

            Ok(values)
        } else {
            Ok(self.head(Some(size))?)
        }
    }

    pub fn head(&self, rows: Option<usize>) -> Result<Vec<Value>, ShellError> {
        let to_row = rows.unwrap_or(5);
        let values = self.to_rows(0, to_row)?;

        Ok(values)
    }

    pub fn tail(&self, rows: Option<usize>) -> Result<Vec<Value>, ShellError> {
        let df = &self.0;
        let to_row = df.height();
        let size = rows.unwrap_or(5);
        let from_row = to_row.saturating_sub(size);

        let values = self.to_rows(from_row, to_row)?;

        Ok(values)
    }

    pub fn to_rows(&self, from_row: usize, to_row: usize) -> Result<Vec<Value>, ShellError> {
        let df = &self.0;
        let upper_row = to_row.min(df.height());

        let mut size: usize = 0;
        let columns = self
            .0
            .get_columns()
            .iter()
            .map(
                |col| match conversion::create_column(col, from_row, upper_row) {
                    Ok(col) => {
                        size = col.len();
                        Ok(col)
                    }
                    Err(e) => Err(e),
                },
            )
            .collect::<Result<Vec<Column>, ShellError>>()?;

        let mut iterators = columns
            .into_iter()
            .map(|col| (col.name().to_string(), col.into_iter()))
            .collect::<Vec<(String, std::vec::IntoIter<Value>)>>();

        let values = (0..size)
            .into_iter()
            .map(|_| {
                let mut cols = vec![];
                let mut vals = vec![];

                for (name, col) in &mut iterators {
                    cols.push(name.clone());

                    match col.next() {
                        Some(v) => vals.push(v),
                        None => vals.push(Value::Nothing {
                            span: Span::unknown(),
                        }),
                    };
                }

                Value::Record {
                    cols,
                    vals,
                    span: Span::unknown(),
                }
            })
            .collect::<Vec<Value>>();

        Ok(values)
    }
}
