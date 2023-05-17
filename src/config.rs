use std::path::PathBuf;
use regex::Regex;
use crate::field::Field;
use crate::order::Order;

#[derive(Clone)]
pub(crate) struct Config {
    tmp: PathBuf,
    tmp_prefix: String,
    tmp_suffix: String,
    tasks: usize,
    queue_size: usize,
    field_separator: char,
    ignore_empty: bool,
    ignore_lines: Option<Regex>,
    concurrent_merge: bool,
    chunk_size_bytes: u64,
    files: usize,
    fields: Vec<Field>,
    order: Order,
    prefix: Vec<String>,
    suffix: Vec<String>,
}

impl Config {
    pub(crate) fn new(
        tmp: PathBuf,
        tmp_prefix: String,
        tmp_suffix: String,
        tasks: usize,
        field_separator: char,
        ignore_empty: bool,
        ignore_lines: Option<Regex>,
        concurrent_merge: bool,
        chunk_size_bytes: u64,
        files: usize,
        fields: Vec<Field>,
        order: Order,
        prefix: Vec<String>,
        suffix: Vec<String>,
    ) -> Config {
        let queue_size = 4096;
        Config {
            tmp,
            tmp_prefix,
            tmp_suffix,
            tasks,
            queue_size,
            field_separator,
            ignore_empty,
            ignore_lines,
            concurrent_merge,
            chunk_size_bytes,
            files,
            fields,
            order,
            prefix,
            suffix,
        }
    }

    pub(crate) fn tmp(&self) -> &PathBuf {
        &self.tmp
    }

    pub(crate) fn tmp_prefix(&self) -> &String {
        &self.tmp_prefix
    }

    pub(crate) fn tmp_suffix(&self) -> &String {
        &self.tmp_suffix
    }

    pub(crate) fn tasks(&self) -> usize {
        self.tasks
    }

    pub(crate) fn queue_size(&self) -> usize {
        self.queue_size
    }

    pub(crate) fn field_separator(&self) -> char {
        self.field_separator
    }

    pub(crate) fn ignore_empty(&self) -> bool {
        self.ignore_empty
    }

    pub(crate) fn ignore_lines(&self) -> &Option<Regex> {
        &self.ignore_lines
    }

    pub(crate) fn concurrent_merge(&self) -> bool {
        self.concurrent_merge
    }

    pub(crate) fn chunk_size_bytes(&self) -> u64 {
        self.chunk_size_bytes
    }

    pub(crate) fn files(&self) -> usize {
        self.files
    }

    pub(crate) fn fields(&self) -> &Vec<Field> {
        &self.fields
    }

    pub(crate) fn order(&self) -> &Order {
        &self.order
    }

    pub(crate) fn prefix(&self) -> &Vec<String> {
        &self.prefix
    }

    pub(crate) fn suffix(&self) -> &Vec<String> {
        &self.suffix
    }
}