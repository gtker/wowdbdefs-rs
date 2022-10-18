use crate::error::ConversionError;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RawDbdFile {
    pub name: String,
    pub columns: HashMap<String, RawColumn>,
    pub definitions: Vec<RawDefinition>,
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

impl RawDbdFile {
    pub fn specific_version(&self, version: &Version) -> Option<&RawDefinition> {
        self.definitions
            .iter()
            .find(|a| compare_versions(version, &a.version_ranges, &a.versions))
    }

    pub fn into_proper(self) -> Result<DbdFile, ConversionError> {
        let mut definitions = Vec::with_capacity(self.definitions.len());

        for def in self.definitions {
            definitions.push(def.to_definition(&self.columns)?)
        }

        Ok(DbdFile {
            name: self.name,
            definitions,
        })
    }

    pub fn find_column(&self, entry: &RawEntry) -> Option<&RawColumn> {
        self.columns.get(&entry.name)
    }

    pub(crate) fn empty(name: String) -> Self {
        Self {
            name,
            columns: HashMap::new(),
            definitions: vec![],
        }
    }
    pub(crate) fn add_column(&mut self, column: RawColumn) {
        self.columns.insert(column.name.clone(), column);
    }

    pub(crate) fn add_database(&mut self, definition: RawDefinition) {
        self.definitions.push(definition);
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RawType {
    Int,
    Float,
    LocString,
    String,
}

impl Display for RawType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RawType::Int => "int",
            RawType::Float => "float",
            RawType::LocString => "locstring",
            RawType::String => "string",
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
pub struct RawColumn {
    pub name: String,
    pub ty: RawType,

    pub foreign_key: Option<ForeignKey>,
    pub verified: bool,

    pub comment: Option<String>,
}

impl RawColumn {
    pub const fn new(
        name: String,
        ty: RawType,
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
pub struct RawEntry {
    pub name: String,
    pub comment: Option<String>,
    pub integer_width: Option<u8>,
    pub array_size: Option<usize>,
    pub unsigned: bool,
    pub primary_key: bool,
    pub inline: bool,
    pub relation: bool,
}

impl RawEntry {
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
pub struct RawDefinition {
    pub versions: BTreeSet<Version>,
    pub version_ranges: Vec<VersionRange>,
    pub layouts: BTreeSet<Layout>,
    pub entries: Vec<RawEntry>,
}

impl RawDefinition {
    pub fn new(
        versions: BTreeSet<Version>,
        version_ranges: Vec<VersionRange>,
        layouts: BTreeSet<Layout>,
        entries: Vec<RawEntry>,
    ) -> Self {
        Self {
            versions,
            version_ranges,
            entries,
            layouts,
        }
    }

    pub fn to_definition(
        &self,
        columns: &HashMap<String, RawColumn>,
    ) -> Result<Definition, ConversionError> {
        let mut entries = Vec::with_capacity(self.entries.len());

        for entry in &self.entries {
            let column = if let Some(c) = columns.get(&entry.name) {
                c
            } else {
                return Err(ConversionError::ColumnNotFound(entry.name.clone()));
            };

            let mut ty = match column.ty {
                RawType::Int => match entry.integer_width {
                    None => return Err(ConversionError::NoIntegerWidth),
                    Some(v) => match entry.unsigned {
                        true => match v {
                            8 => Type::UInt8,
                            16 => Type::UInt16,
                            32 => Type::UInt32,
                            64 => Type::UInt64,
                            v => return Err(ConversionError::InvalidIntegerWidth(v.into())),
                        },
                        false => match v {
                            8 => Type::Int8,
                            16 => Type::Int16,
                            32 => Type::Int32,
                            64 => Type::Int64,
                            v => return Err(ConversionError::InvalidIntegerWidth(v.into())),
                        },
                    },
                },
                RawType::Float => Type::Float,
                RawType::LocString => Type::LocString,
                RawType::String => Type::String,
            };

            if let Some(foreign_key) = &column.foreign_key {
                match ty {
                    Type::Array { .. }
                    | Type::Int8
                    | Type::Int16
                    | Type::Int32
                    | Type::Int64
                    | Type::UInt8
                    | Type::UInt16
                    | Type::UInt32
                    | Type::UInt64 => {
                        ty = Type::ForeignKey {
                            ty: Box::new(ty),
                            key: foreign_key.clone(),
                        };
                    }
                    Type::Float => return Err(ConversionError::FloatAsForeignKey),
                    Type::LocString => return Err(ConversionError::LocStringAsForeignKey),
                    Type::String => return Err(ConversionError::StringAsForeignKey),
                    Type::ForeignKey { .. } => {
                        unreachable!("ty has not been set to foreign key yet")
                    }
                }
            }

            if let Some(width) = entry.array_size {
                ty = Type::Array {
                    ty: Box::new(ty.clone()),
                    width,
                };
            }

            entries.push(Entry {
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

        Ok(Definition {
            versions: self.versions.clone(),
            version_ranges: self.version_ranges.clone(),
            layouts: self.layouts.clone(),
            entries,
        })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default)]
pub struct Definition {
    pub versions: BTreeSet<Version>,
    pub version_ranges: Vec<VersionRange>,
    pub layouts: BTreeSet<Layout>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Entry {
    pub name: String,

    pub ty: Type,

    pub comment: Option<String>,
    pub column_comment: Option<String>,

    pub verified: bool,
    pub primary_key: bool,
    pub inline: bool,
    pub relation: bool,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Type {
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

    ForeignKey { ty: Box<Type>, key: ForeignKey },

    Array { ty: Box<Type>, width: usize },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DbdFile {
    pub name: String,
    pub definitions: Vec<Definition>,
}

impl DbdFile {
    pub fn specific_version(&self, version: &Version) -> Option<&Definition> {
        self.definitions
            .iter()
            .find(|a| compare_versions(version, &a.version_ranges, &a.versions))
    }
}
