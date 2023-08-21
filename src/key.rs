use std::cmp::Ordering;
use std::str::FromStr;

use data_encoding::HEXLOWER;

use crate::field::Field;
use crate::field_type::FieldType;

#[derive(Debug)]
pub(crate) enum Key {
    String {
        s: String
    },
    Integer {
        i: i64
    },
    Number {
        n: f64
    },
}

impl Key {
    pub(crate) fn new(field: &str, field_def: &Field) -> Result<Key, anyhow::Error> {
        match field_def.field_type() {
            FieldType::String => {
                let mut key = field.to_string();
                if field_def.ignore_blanks() {
                    key = key.trim().to_string();
                }

                if field_def.ignore_case() {
                    key = key.to_uppercase()
                }

                if field_def.random() {
                    key = HEXLOWER.encode(&rand::random::<[u8; 16]>());
                }

                Ok(
                    Key::String {
                        s: key
                    }
                )
            }
            FieldType::Integer => {
                let mut key = i64::from_str(field.trim())?;
                if field_def.random() {
                    key = rand::random::<i64>()
                }

                Ok(
                    Key::Integer {
                        i: key
                    }
                )
            }
            FieldType::Number => {
                let mut key = f64::from_str(field.trim())?;
                if field_def.random() {
                    key = rand::random::<f64>()
                }

                Ok(
                    Key::Number {
                        n: key
                    }
                )
            }
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Key::String { s } => { Some(s.as_str()) }
            Key::Integer { .. } => {
                None
            }
            Key::Number { .. } => {
                None
            }
        }
    }

    fn as_integer(&self) -> Option<i64> {
        match self {
            Key::String { .. } => {
                None
            }
            Key::Integer { i } => {
                Some(*i)
            }
            Key::Number { .. } => {
                None
            }
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Key::String { .. } => {
                None
            }
            Key::Integer { .. } => {
                None
            }
            Key::Number { n } => {
                Some(*n)
            }
        }
    }
}

impl Eq for Key {}

impl PartialEq<Self> for Key {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Key::String { s } => { s.eq(other.as_str().unwrap()) }
            Key::Integer { i } => { i.eq(&other.as_integer().unwrap()) }
            Key::Number { n } => { n.eq(&other.as_number().unwrap()) }
        }
    }
}

impl PartialOrd<Self> for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Key::String { s } => { s.as_str().cmp(other.as_str().unwrap()) }
            Key::Integer { i } => { i.cmp(&other.as_integer().unwrap()) }
            Key::Number { n } => {
                if n.is_nan() && other.as_number().unwrap().is_nan() {
                    Ordering::Equal
                } else if !n.is_nan() && other.as_number().unwrap().is_nan() {
                    Ordering::Greater
                } else if n.is_nan() && !other.as_number().unwrap().is_nan() {
                    Ordering::Less
                } else {
                    n.partial_cmp(&other.as_number().unwrap()).unwrap()
                }
            }
        }
    }
}