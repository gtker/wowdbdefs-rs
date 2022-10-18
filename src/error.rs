//! Error types for the crate.
//!
use std::fmt::{Display, Formatter};

/// Main error for parsing the files.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ParseError {
    pub column: usize,
    pub line: usize,
    pub reason: DbdErrorReason,
}

impl ParseError {
    pub(crate) const fn new(column: usize, line: usize, reason: DbdErrorReason) -> Self {
        Self {
            column,
            line,
            reason,
        }
    }

    /// Prints `contents` from the `(line, column)` to the end of the string.
    pub fn start_str_at<'a>(&self, mut contents: &'a str) -> Option<&'a str> {
        let mut i = 0_usize;

        if self.line == 0 {
            return Some(&contents[self.column..]);
        }

        while let Some((_, b)) = contents.split_once('\n') {
            i += 1;

            if self.line == i {
                return Some(&b[self.column..]);
            }

            contents = b;
        }

        None
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Column {}, line {}: {}",
            self.column, self.line, self.reason,
        ))
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DbdErrorReason {
    NoSpaceInColumn,

    NoDoubleColonInForeignKey,
    NoClosingForeignKeyAngleBracket,

    NoClosingAnnotationDollarSign,

    NoClosingIntegerSizeAngleBracket,
    InvalidIntegerSizeNumber(String),

    NoClosingArraySizeSquareBracket,
    InvalidArraySizeNumber(String),

    InvalidLayout(String),
    InvalidBuild(String),

    InvalidType(String),
}

impl Display for DbdErrorReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DbdErrorReason::NoSpaceInColumn => "no space to separate column name and type",
            DbdErrorReason::NoDoubleColonInForeignKey => {
                "no '::' inside foreign key '<' and '>' brackets"
            }
            DbdErrorReason::NoClosingForeignKeyAngleBracket => {
                "no matching '>' for opening '<' in foreign key"
            }
            DbdErrorReason::NoClosingAnnotationDollarSign => {
                "no matching '$' for opening '$' in annotations"
            }
            DbdErrorReason::NoClosingArraySizeSquareBracket => {
                "no matching ']' for opening '[' in array"
            }
            DbdErrorReason::NoClosingIntegerSizeAngleBracket => {
                "no matching '>' for opening '<' in integer size"
            }
            DbdErrorReason::InvalidLayout(s) => {
                return f.write_fmt(format_args!("invalid hex string for layout: '{}'", s));
            }
            DbdErrorReason::InvalidIntegerSizeNumber(s) => {
                return f.write_fmt(format_args!("invalid integer size: '{}'", s));
            }
            DbdErrorReason::InvalidArraySizeNumber(s) => {
                return f.write_fmt(format_args!("invalid array size: '{}'", s));
            }
            DbdErrorReason::InvalidType(s) => {
                return f.write_fmt(format_args!("invalid type name: '{}'", s));
            }
            DbdErrorReason::InvalidBuild(s) => {
                return f.write_fmt(format_args!("invalid build format: '{}'", s));
            }
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ConversionError {
    InvalidIntegerWidth(usize),
    NoIntegerWidth,

    ColumnNotFound(String),

    LocStringAsForeignKey,
    StringAsForeignKey,
    FloatAsForeignKey,
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::InvalidIntegerWidth(i) => {
                write!(f, "invalid integer size '{}'", i)
            }
            ConversionError::NoIntegerWidth => write!(f, "no integer width for integer"),
            ConversionError::ColumnNotFound(s) => write!(f, "column not found '{}'", s),
            ConversionError::LocStringAsForeignKey => {
                write!(f, "LocString type is set as foreign key")
            }
            ConversionError::StringAsForeignKey => {
                write!(f, "String type is set as foreign key")
            }
            ConversionError::FloatAsForeignKey => {
                write!(f, "Float type is set as foreign key")
            }
        }
    }
}

impl std::error::Error for ConversionError {}
