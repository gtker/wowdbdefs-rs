use crate::error::SpecificConversionError;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DbdFile {
    pub name: String,
    pub columns: HashMap<String, Column>,
    pub definitions: Vec<Definition>,
}

fn compare_versions(
    version: &Version,
    version_ranges: &[VersionRange],
    versions: &BTreeSet<Version>,
) -> bool {
    for b in version_ranges {
        if b.within_range(version) {
            return true;
        }
    }

    for b in versions {
        if b == version {
            return true;
        }
    }

    false
}

impl DbdFile {
    pub fn specific_version(&self, version: &Version) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|a| compare_versions(version, &a.version_ranges, &a.versions))
    }

    pub fn into_specific(self) -> Result<SpecificDbdFile, SpecificConversionError> {
        let mut definitions = Vec::with_capacity(self.definitions.len());

        for def in self.definitions {
            definitions.push(def.to_specific(&self.columns)?)
        }

        Ok(SpecificDbdFile {
            name: self.name,
            definitions,
        })
    }

    pub fn find_column(&self, entry: &Entry) -> Option<&Column> {
        self.columns.get(&entry.name)
    }

    pub(crate) fn empty(name: String) -> Self {
        Self {
            name,
            columns: HashMap::new(),
            definitions: vec![],
        }
    }
    pub(crate) fn add_column(&mut self, column: Column) {
        self.columns.insert(column.name.clone(), column);
    }

    pub(crate) fn add_database(&mut self, definition: Definition) {
        self.definitions.push(definition);
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Float,
    LocString,
    String,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Type::Int => "int",
            Type::Float => "float",
            Type::LocString => "locstring",
            Type::String => "string",
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ForeignKey {
    pub database: String,
    pub column: String,
}

impl Display for ForeignKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}::{}>", self.database, self.column)
    }
}

impl ForeignKey {
    pub const fn new(database: String, column: String) -> Self {
        Self { database, column }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Column {
    pub name: String,
    pub ty: Type,

    pub foreign_key: Option<ForeignKey>,
    pub verified: bool,

    pub comment: Option<String>,
}

impl Column {
    pub const fn new(
        name: String,
        ty: Type,
        foreign_key: Option<ForeignKey>,
        verified: bool,
        comment: Option<String>,
    ) -> Self {
        Self {
            name,
            ty,
            foreign_key,
            verified,
            comment,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Default, PartialEq, Eq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u16,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => match self.patch.cmp(&other.patch) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => self.build.cmp(&other.build),
                },
            },
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}.{}",
            self.major, self.minor, self.patch, self.build
        ))
    }
}

impl Version {
    pub const fn new(major: u8, minor: u8, patch: u8, build: u16) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct VersionRange {
    pub from: Version,
    pub to: Version,
}

impl VersionRange {
    pub const fn new(from: Version, to: Version) -> Self {
        Self { from, to }
    }

    pub fn within_range(&self, version: &Version) -> bool {
        self.from <= *version && self.to >= *version
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Layout {
    pub inner: u32,
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

impl Layout {
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Entry {
    pub name: String,
    pub comment: Option<String>,
    pub integer_width: Option<u8>,
    pub array_size: Option<usize>,
    pub unsigned: bool,
    pub primary_key: bool,
    pub inline: bool,
    pub relation: bool,
}

impl Entry {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        name: String,
        comment: Option<String>,
        integer_width: Option<u8>,
        array_size: Option<usize>,
        unsigned: bool,
        primary_key: bool,
        inline: bool,
        relation: bool,
    ) -> Self {
        Self {
            name,
            comment,
            integer_width,
            unsigned,
            array_size,
            primary_key,
            inline,
            relation,
        }
    }

    pub const fn has_any_tag(&self) -> bool {
        self.primary_key || !self.inline || self.relation
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Definition {
    pub versions: BTreeSet<Version>,
    pub version_ranges: Vec<VersionRange>,
    pub layouts: BTreeSet<Layout>,
    pub entries: Vec<Entry>,
}

impl Definition {
    pub fn new(
        versions: BTreeSet<Version>,
        version_ranges: Vec<VersionRange>,
        layouts: BTreeSet<Layout>,
        entries: Vec<Entry>,
    ) -> Self {
        Self {
            versions,
            version_ranges,
            entries,
            layouts,
        }
    }

    pub fn to_specific(
        &self,
        columns: &HashMap<String, Column>,
    ) -> Result<SpecificDefinition, SpecificConversionError> {
        let mut entries = Vec::with_capacity(self.entries.len());

        for entry in &self.entries {
            let column = if let Some(c) = columns.get(&entry.name) {
                c
            } else {
                return Err(SpecificConversionError::ColumnNotFound(entry.name.clone()));
            };

            let mut ty = match column.ty {
                Type::Int => match entry.integer_width {
                    None => return Err(SpecificConversionError::NoIntegerWidth),
                    Some(v) => match entry.unsigned {
                        true => match v {
                            8 => SpecificType::UInt8,
                            16 => SpecificType::UInt16,
                            32 => SpecificType::UInt32,
                            64 => SpecificType::UInt64,
                            v => {
                                return Err(SpecificConversionError::InvalidIntegerWidth(v.into()))
                            }
                        },
                        false => match v {
                            8 => SpecificType::Int8,
                            16 => SpecificType::Int16,
                            32 => SpecificType::Int32,
                            64 => SpecificType::Int64,
                            v => {
                                return Err(SpecificConversionError::InvalidIntegerWidth(v.into()))
                            }
                        },
                    },
                },
                Type::Float => SpecificType::Float,
                Type::LocString => SpecificType::LocString,
                Type::String => SpecificType::String,
            };

            if let Some(foreign_key) = &column.foreign_key {
                match ty {
                    SpecificType::Array { .. }
                    | SpecificType::Int8
                    | SpecificType::Int16
                    | SpecificType::Int32
                    | SpecificType::Int64
                    | SpecificType::UInt8
                    | SpecificType::UInt16
                    | SpecificType::UInt32
                    | SpecificType::UInt64 => {
                        ty = SpecificType::ForeignKey {
                            ty: Box::new(ty),
                            key: foreign_key.clone(),
                        };
                    }
                    SpecificType::Float => return Err(SpecificConversionError::FloatAsForeignKey),
                    SpecificType::LocString => {
                        return Err(SpecificConversionError::LocStringAsForeignKey)
                    }
                    SpecificType::String => {
                        return Err(SpecificConversionError::StringAsForeignKey)
                    }
                    SpecificType::ForeignKey { .. } => {
                        unreachable!("ty has not been set to foreign key yet")
                    }
                }
            }

            if let Some(width) = entry.array_size {
                ty = SpecificType::Array {
                    ty: Box::new(ty.clone()),
                    width,
                };
            }

            entries.push(SpecificEntry {
                name: entry.name.clone(),
                ty,
                comment: entry.comment.clone(),
                column_comment: column.comment.clone(),
                verified: column.verified,
                primary_key: entry.primary_key,
                inline: entry.inline,
                relation: entry.relation,
            });
        }

        Ok(SpecificDefinition {
            versions: self.versions.clone(),
            version_ranges: self.version_ranges.clone(),
            layouts: self.layouts.clone(),
            entries,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct SpecificDefinition {
    pub versions: BTreeSet<Version>,
    pub version_ranges: Vec<VersionRange>,
    pub layouts: BTreeSet<Layout>,
    pub entries: Vec<SpecificEntry>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SpecificEntry {
    pub name: String,

    pub ty: SpecificType,

    pub comment: Option<String>,
    pub column_comment: Option<String>,

    pub verified: bool,
    pub primary_key: bool,
    pub inline: bool,
    pub relation: bool,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SpecificType {
    Int8,
    Int16,
    Int32,
    Int64,

    UInt8,
    UInt16,
    UInt32,
    UInt64,

    Float,
    LocString,
    String,

    ForeignKey {
        ty: Box<SpecificType>,
        key: ForeignKey,
    },

    Array {
        ty: Box<SpecificType>,
        width: usize,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SpecificDbdFile {
    pub name: String,
    pub definitions: Vec<SpecificDefinition>,
}

impl SpecificDbdFile {
    pub fn specific_version(&self, version: &Version) -> Option<&SpecificDefinition> {
        self.definitions
            .iter()
            .find(|a| compare_versions(version, &a.version_ranges, &a.versions))
    }
}
