use std::cmp::Ordering;

use anyhow::anyhow;

use crate::field::Field;
use crate::key::Key;
use crate::order::Order;

#[derive(Debug)]
pub(crate) struct LineRecord {
    line: String,
    keys: Vec<Key>,
    order: Order,
}

impl LineRecord {
    pub(crate) fn new(line: String, fields: &Vec<Field>, field_separator: char, order: Order) -> Result<LineRecord, anyhow::Error> {
        if fields.len() == 1 && fields[0].index() == 0 {
            let field = &fields[0];
            let key = Key::new(line.as_str(), field).or_else(
                |e| Err(anyhow!("line: {line}, error: {e}"))
            )?;

            Ok(
                LineRecord {
                    line,
                    keys: vec![key],
                    order,
                }
            )
        } else {
            let mut keys: Vec<Key> = Vec::new();
            let parts: Vec<&str> = line.split(field_separator).into_iter().collect();
            let mut error = None;
            for field in fields {
                if field.index() == 0 {
                    error = Some(
                        anyhow!(
                            "Field index of 0 must be specified only once, meaning the entire line is to be used as a key".to_string()
                        )
                    );
                    break;
                }
                if field.index() > parts.len() {
                    error = Some(
                        anyhow!(
                            "Requested comparison for field {} but there are only {} fields using {} as field separator.",
                            field.index(),
                            parts.len(),
                            field_separator,
                       )
                    );
                    break;
                }
                keys.push(Key::new(parts[field.index() - 1], field)?)
            }
            if let Some(e) = error {
                Err(anyhow!("line: {line}, error: {e}"))
            } else {
                Ok(
                    LineRecord {
                        line,
                        keys,
                        order,
                    }
                )
            }
        }
    }

    pub fn line(self) -> String {
        self.line
    }
}

impl Eq for LineRecord {}

impl PartialEq<Self> for LineRecord {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys
    }
}

impl PartialOrd<Self> for LineRecord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineRecord {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.keys.cmp(&other.keys);
        match ordering {
            Ordering::Less => {
                match &self.order {
                    Order::Asc => {
                        Ordering::Less
                    }
                    Order::Desc => {
                        Ordering::Greater
                    }
                }
            }
            Ordering::Equal => {
                Ordering::Equal
            }
            Ordering::Greater => {
                match &self.order {
                    Order::Asc => {
                        Ordering::Greater
                    }
                    Order::Desc => {
                        Ordering::Less
                    }
                }
            }
        }
    }
}

