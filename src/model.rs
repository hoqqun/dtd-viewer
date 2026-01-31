use indexmap::IndexMap;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct Dtd {
    pub elements: IndexMap<String, Element>,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Element {
    pub name: String,
    pub content: ContentModel,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum ContentModel {
    Empty,
    Any,
    Pcdata,
    Mixed(Vec<String>),
    Children(Group),
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub kind: GroupKind,
    pub items: Vec<GroupItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum GroupKind {
    Sequence,
    Choice,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupItem {
    pub content: GroupItemContent,
    pub quantifier: Quantifier,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum GroupItemContent {
    Name(String),
    Group(Group),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Quantifier {
    One,
    Optional,
    ZeroOrMore,
    OneOrMore,
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quantifier::One => Ok(()),
            Quantifier::Optional => write!(f, "?"),
            Quantifier::ZeroOrMore => write!(f, "*"),
            Quantifier::OneOrMore => write!(f, "+"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Attribute {
    pub name: String,
    pub attr_type: AttrType,
    pub default: AttrDefault,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "values")]
pub enum AttrType {
    Cdata,
    Id,
    Idref,
    Idrefs,
    Nmtoken,
    Nmtokens,
    Enumeration(Vec<String>),
}

impl fmt::Display for AttrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrType::Cdata => write!(f, "CDATA"),
            AttrType::Id => write!(f, "ID"),
            AttrType::Idref => write!(f, "IDREF"),
            AttrType::Idrefs => write!(f, "IDREFS"),
            AttrType::Nmtoken => write!(f, "NMTOKEN"),
            AttrType::Nmtokens => write!(f, "NMTOKENS"),
            AttrType::Enumeration(vals) => write!(f, "({})", vals.join("|")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum AttrDefault {
    Required,
    Implied,
    Fixed(String),
    Default(String),
}

impl fmt::Display for AttrDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttrDefault::Required => write!(f, "#REQUIRED"),
            AttrDefault::Implied => write!(f, "#IMPLIED"),
            AttrDefault::Fixed(v) => write!(f, "#FIXED \"{}\"", v),
            AttrDefault::Default(v) => write!(f, "\"{}\"", v),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Entity {
    pub name: String,
    pub is_parameter: bool,
    pub kind: EntityKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum EntityKind {
    Internal { value: String },
    ExternalSystem { uri: String },
    ExternalPublic { public_id: String, uri: String },
}
