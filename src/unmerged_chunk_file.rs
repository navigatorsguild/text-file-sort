use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::Context;

use crate::field::Field;
use crate::line_record::LineRecord;
use crate::order::Order;

#[derive(Debug)]
pub(crate) struct UnmergedChunkFile {
    path: PathBuf,
    reader: BufReader<File>,
    head: Option<LineRecord>,
    fields: Vec<Field>,
    field_separator: char,
    order: Order,
}

impl UnmergedChunkFile {
    pub(crate) fn new(path: PathBuf, fields: &Vec<Field>, field_separator: char, order: Order) -> Result<UnmergedChunkFile, anyhow::Error> {
        let file = File::open(path.clone()).with_context(|| format!("path: {}", path.to_string_lossy()))?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes > 0 {
            Ok(
                UnmergedChunkFile {
                    path,
                    reader,
                    head: Some(LineRecord::new(line, fields, field_separator, order.clone())?),
                    fields: fields.clone(),
                    field_separator,
                    order,
                }
            )
        } else {
            Ok(
                UnmergedChunkFile {
                    path,
                    reader,
                    head: None,
                    fields: fields.clone(),
                    field_separator,
                    order,
                }
            )
        }
    }

    pub(crate) fn line_record(&mut self) -> Option<LineRecord> {
        let mut line = String::new();
        let bytes = self.reader.read_line(&mut line).ok()?;
        let line_record = if bytes > 0 {
            LineRecord::new(line, &self.fields, self.field_separator, self.order.clone()).ok()
        } else {
            None
        };
        std::mem::replace(&mut self.head, line_record)
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Eq for UnmergedChunkFile {}

impl PartialEq<Self> for UnmergedChunkFile {
    fn eq(&self, other: &Self) -> bool {
        if self.head.is_none() && other.head.is_none() {
            true
        } else if self.head.is_none() || other.head.is_none() {
            false
        } else {
            other.head.as_ref().unwrap().eq(&self.head.as_ref().unwrap())
        }
    }
}

impl PartialOrd<Self> for UnmergedChunkFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnmergedChunkFile {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.head.is_none() && other.head.is_none() {
            Ordering::Equal
        } else if self.head.is_none() && other.head.is_some() {
            // none > some so empty files will pop from BinaryHeap first
            Ordering::Greater
        } else if self.head.is_some() && other.head.is_none() {
            Ordering::Less
        } else {
            other.head.as_ref().unwrap().cmp(&self.head.as_ref().unwrap())
        }
    }
}