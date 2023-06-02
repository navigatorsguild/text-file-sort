use std::cell::RefCell;
use std::cmp::{max, Reverse};
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Context};
use command_executor::shutdown_mode::ShutdownMode;
use command_executor::thread_pool::ThreadPool;
use command_executor::thread_pool_builder::ThreadPoolBuilder;
use rand::distributions::uniform::SampleBorrow;
use regex::Regex;
use rlimit::{getrlimit, Resource, setrlimit};
use tempfile::{Builder, NamedTempFile};

use crate::chunk_iterator::ChunkIterator;
use crate::config::Config;
use crate::field::Field;
use crate::field_type::FieldType;
use crate::line_record::LineRecord;
use crate::order::Order;
use crate::sort_command::SortCommand;
use crate::sorted_chunk_file::SortedChunkFile;
use crate::unmerged_chunk_file::UnmergedChunkFile;

thread_local! {
    pub(crate) static LINE_CAPACITY: RefCell<usize> = RefCell::new(1);
    pub(crate) static LINE_RECORDS_CAPACITY: RefCell<usize> = RefCell::new(1);
    pub(crate) static SORTED_FILES: RefCell<BinaryHeap<Reverse<SortedChunkFile>>> = RefCell::new(BinaryHeap::new());
    pub(crate) static CONFIG: RefCell<Option<Config>> = RefCell::new(None);
}

pub(crate) fn get_line_capacity() -> usize {
    LINE_CAPACITY.with(|capacity| *capacity.borrow().borrow())
}

pub(crate) fn set_line_capacity(value: usize) {
    LINE_CAPACITY.with(|capacity| capacity.replace(value));
}

pub(crate) fn get_line_records_capacity() -> usize {
    LINE_RECORDS_CAPACITY.with(|capacity| *capacity.borrow().borrow())
}

pub(crate) fn set_line_records_capacity(value: usize) {
    LINE_RECORDS_CAPACITY.with(|capacity| capacity.replace(value));
}

pub(crate) fn get_tl_config() -> Config {
    CONFIG.with(
        |config| {
            config.borrow().as_ref().unwrap().clone()
        }
    )
}

pub(crate) fn create_tmp_file(config: &Config) -> NamedTempFile {
    Builder::new()
        .prefix(config.tmp_prefix())
        .suffix(config.tmp_suffix())
        .tempfile_in(config.tmp())
        .or_else(|e| Err(anyhow!("Failed to create new temp file: {}", e.to_string())))
        .unwrap()
}

/// Sort a text file with record like lines
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use text_file_sort::sort::Sort;
///
/// // parallel record sort
/// fn sort_records(input: PathBuf, output: PathBuf, tmp: PathBuf) -> Result<(), anyhow::Error> {
///    let mut text_file_sort = Sort::new(vec![input.clone()], output.clone());
///     // set number of CPU cores the sort will attempt to use. When given the number that exceeds
///     // the number of available CPU cores the work will be split among available cores with
///     // somewhat degraded performance.
///     text_file_sort.with_tasks(2);
///     // set the directory for intermediate results. The default is the system temp dir -
///     // std::env::temp_dir(), however, for large files it is recommended to provide a dedicated
///     // directory for intermediate files, preferably on the same file system as the output result.
///     text_file_sort.with_tmp_dir(tmp);
///     text_file_sort.sort()
/// }
/// ```
pub struct Sort {
    input_files: Vec<PathBuf>,
    output: PathBuf,
    tmp: PathBuf,
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
    endl: char,
}

impl Sort {
    /// Create a default Sort definition.
    ///
    /// A default Sort definition will use the system temporary
    /// directory as defined by std::env::temp_dir().
    /// * The default field separator is a TAB ('\t')
    /// * The complete line will be considered as a single String field
    /// * empty lines will be sorted lexicographically
    /// * lines starting with '#' will be ignored
    /// * max intermediate files is set to 1024.
    /// * input is read in chunks of 10 MB bytes
    /// * default Order is Asc
    /// * prefix and suffix are empty
    /// * default end lines is '\n'
    ///
    /// The Sort implementation will increase the file descriptor rlimit to accommodate configured
    /// open files
    pub fn new(input_files: Vec<PathBuf>, output: PathBuf) -> Sort {
        Sort {
            input_files,
            output,
            tmp: std::env::temp_dir(),
            tasks: 0,
            field_separator: '\t',
            ignore_empty: false,
            ignore_lines: Some(Regex::new("^#").unwrap()),
            concurrent_merge: true,
            chunk_size_bytes: 10_000_000,
            files: 1024,
            fields: vec![],
            order: Order::Asc,
            prefix: vec![],
            suffix: vec![],
            endl: '\n',
        }
    }

    /// Set directory for intermediate files. By default use std::env::temp_dir()
    /// It is recommended for large files to create a dedicated directory for intermediate files
    /// on the same file system as the output target
    pub fn with_tmp_dir(&mut self, tmp: PathBuf) {
        self.tmp = tmp;
    }

    /// Set the number of tasks. The default is zero which will result in using all system cores
    pub fn with_tasks(&mut self, tasks: usize) {
        self.tasks = tasks;
    }

    /// Set the field separator. The default is '\t'
    pub fn with_field_separator(&mut self, field_separator: char) {
        self.field_separator = field_separator
    }

    /// Merge sorted files concurrently to reduce the number of files before the final merge
    pub fn with_concurrent_merge(&mut self, concurrent_merge: bool) {
        self.concurrent_merge = concurrent_merge
    }

    /// The input will be read in chunks of 'chunk_size_bytes' respecting line boundaries
    pub fn with_chunk_size_bytes(&mut self, chunk_size_bytes: u64) {
        self.chunk_size_bytes = chunk_size_bytes;
    }

    /// The input will be read in chunks of 'chunk_size_mb' MB respecting line boundaries
    pub fn with_chunk_size_mb(&mut self, chunk_size_mb: u64) {
        self.chunk_size_bytes = chunk_size_mb * 1_000_000;
    }

    /// Set the number of intermediate files. The default is 1024.
    pub fn with_intermediate_files(&mut self, files: usize) {
        self.files = files;
    }

    /// Direct the algorithm to ignore empty lines. The default is false
    pub fn with_ignore_empty(&mut self) {
        self.ignore_empty = true;
    }

    /// Specify which lines to ignore. Each line matching the regex will be ignored and will not
    /// appear in the output.
    pub fn with_ignore_lines(&mut self, r: Regex) {
        self.ignore_lines = Some(r)
    }

    /// Add field specification. The default is to treat the complete line as a single String
    /// field in the record
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Replace all fields with the `fields` value.
    pub fn with_fields(&mut self, fields: Vec<Field>) {
        self.fields = fields
    }

    /// Set [Order]
    pub fn with_order(&mut self, order: Order) {
        self.order = order
    }

    /// Add file prefix. The provided prefix will be inserted at the beginning of the sorted file
    pub fn add_prefix_line(&mut self, prefix_line: String) {
        self.prefix.push(prefix_line);
    }

    /// Set prefix lines
    pub fn with_prefix_lines(&mut self, prefix_lines: Vec<String>) {
        self.prefix = prefix_lines;
    }

    /// Add file suffix. The provided suffix will be inserted at the end of the sorted file
    pub fn add_suffix_line(&mut self, suffix_line: String) {
        self.suffix.push(suffix_line);
    }

    /// Set suffix lines
    pub fn with_suffix_lines(&mut self, suffix_lines: Vec<String>) {
        self.suffix = suffix_lines;
    }

    /// Set line ending char - not supporting CRLF
    pub fn with_endl(&mut self, endl: char) {
        self.endl = endl
    }

    /// Sort input files or STDIN
    pub fn sort(&self) -> Result<(), anyhow::Error> {
        let config = self.create_config();
        let (current_soft, current_hard) = Self::get_rlimits()?;
        log::info!("Current rlimit NOFILE, soft: {}, hard: {}", current_soft, current_hard);
        let new_soft = max((config.files() + 256) as u64, current_soft);
        log::info!("Set new rlimit NOFILE, soft: {}, hard: {}", new_soft, current_hard);
        Self::set_rlimits(new_soft, current_hard)?;
        Self::internal_sort(&self.input_files, &config, &self.output)?;
        log::info!("Restore rlimit NOFILE, soft: {}, hard: {}", current_soft, current_hard);
        Self::set_rlimits(current_soft, current_hard)?;
        Ok(())
    }

    fn get_rlimits() -> Result<(u64, u64), anyhow::Error> {
        getrlimit(Resource::NOFILE).with_context(|| "getrlimit")
    }

    fn set_rlimits(soft: u64, hard: u64) -> Result<(), anyhow::Error> {
        setrlimit(Resource::NOFILE, soft, hard)
            .with_context(|| format!("set rlimit NOFILE, soft: {}, hard: {}", soft, hard))?;
        Ok(())
    }

    fn create_config(&self) -> Config {
        let fields = if self.fields.len() == 0 {
            vec![Field::new(0, FieldType::String)]
        } else {
            self.fields.clone()
        };

        let mut tasks = self.tasks;
        if self.tasks == 0 {
            tasks = num_cpus::get();
        }

        let mut files = tasks * 2;
        if self.files > files {
            files = self.files
        }

        let config = Config::new(
            self.tmp.clone(),
            "part-".to_string(),
            ".unmerged".to_string(),
            tasks,
            self.field_separator,
            self.ignore_empty,
            self.ignore_lines.clone(),
            self.concurrent_merge,
            self.chunk_size_bytes,
            files,
            fields,
            self.order.clone(),
            self.prefix.clone(),
            self.suffix.clone(),
            self.endl
        );
        config
    }

    fn merge_sorted_files(thread_pool: &ThreadPool) {
        thread_pool.in_all_threads(
            Arc::new(
                || {
                    SORTED_FILES.with(
                        |sorted_files| {
                            if sorted_files.borrow().len() > 1 {
                                let mut intermediate = Vec::new();
                                while sorted_files.borrow().len() > 0 {
                                    let sorted_chunk_file = sorted_files.borrow_mut().pop().unwrap();
                                    let path = sorted_chunk_file.0.path().clone();
                                    intermediate.push(path);
                                }
                                let config = get_tl_config();
                                let (path, size) = Self::internal_merge(intermediate, &config, true, false).expect("TODO: ");
                                sorted_files
                                    .borrow_mut()
                                    .push(Reverse(SortedChunkFile::new(path, size)));
                            }
                        }
                    );
                }
            )
        );
    }

    fn collect_sorted_files(thread_pool: &mut ThreadPool) -> Vec<PathBuf> {
        let result: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
        let result_clone = result.clone();
        thread_pool.in_all_threads_mut(
            Arc::new(
                Mutex::new(
                    move || {
                        SORTED_FILES.with(
                            |sorted_files| {
                                log::info!("Start collecting thread intermediate results, thread: {}", thread::current().name().unwrap_or("unnamed"));
                                let mut intermediate = Vec::new();
                                while sorted_files.borrow().len() > 0 {
                                    let sorted_chunk_file = sorted_files.borrow_mut().pop().unwrap();
                                    let path = sorted_chunk_file.0.path().clone();
                                    intermediate.push(path);
                                }
                                let mut result_guard = result_clone.lock().unwrap();
                                result_guard.append(&mut intermediate);
                                log::info!("Finish collecting thread intermediate results, thread: {}", thread::current().name().unwrap_or("unnamed"));
                            }
                        );
                    }
                )
            )
        );
        let mut result_guard = result.lock().unwrap();
        std::mem::take(result_guard.as_mut())
    }

    pub fn check(&self) -> Result<bool, anyhow::Error> {
        let config = self.create_config();

        let mut result = true;
        for path in &self.input_files {
            result = Self::internal_check(path, &config)?;
            if !result {
                break;
            }
        }
        Ok(result)
    }

    pub(crate) fn internal_check(path: &PathBuf, config: &Config) -> Result<bool, anyhow::Error> {
        let mut result = true;
        let mut line = String::new();
        let mut previous: Option<LineRecord> = None;
        let mut reader = BufReader::new(File::open(path)?);
        while reader.read_line(&mut line)? != 0 {
            if config.ignore_empty() && line.trim().is_empty() {
                continue;
            }

            if let Some(r) = config.ignore_lines() {
                if r.is_match(line.trim()) {
                    continue;
                }
            }
            let current_line_record = LineRecord::new(
                line,
                config.fields(),
                config.field_separator(),
                config.order().clone(),
            )?;

            match previous {
                None => {
                    previous = Some(current_line_record);
                }
                Some(previous_line_record) => {
                    if previous_line_record <= current_line_record {
                        previous = Some(current_line_record);
                    } else {
                        result = false;
                        break;
                    }
                }
            }
            line = String::new();
        }
        Ok(result)
    }

    pub fn merge(&self) -> Result<(), anyhow::Error> {
        let config = self.create_config();
        let (current_soft, current_hard) = Self::get_rlimits()?;
        log::info!("Current rlimit NOFILE, soft: {}, hard: {}", current_soft, current_hard);
        let new_soft = max((config.files() + 256) as u64, current_soft);
        log::info!("Set new rlimit NOFILE, soft: {}, hard: {}", new_soft, current_hard);
        Self::set_rlimits(new_soft, current_hard)?;
        let (path, _lines) = Self::internal_merge(self.input_files.clone(), &config, false, true)?;
        std::fs::rename(path.clone(), &self.output)
            .with_context(|| anyhow!("Rename {} to {}", path.to_string_lossy(), self.output.to_string_lossy()))?;
        log::info!("Restore rlimit NOFILE, soft: {}, hard: {}", current_soft, current_hard);
        Self::set_rlimits(current_soft, current_hard)?;
        Ok(())
    }

    pub(crate) fn internal_merge(files: Vec<PathBuf>, config: &Config, remove_merged: bool, add_prefix_suffix: bool) -> Result<(PathBuf, usize), anyhow::Error> {
        log::info!("Merging {} sorted files, thread: {}", files.len(), thread::current().name().unwrap_or("unnamed"));
        let mut merged_len: usize = 0;
        let merged_file = create_tmp_file(config);
        let (persisted_merged_file, path) = merged_file.keep()?;
        let mut merged_writer = BufWriter::new(persisted_merged_file);
        if add_prefix_suffix {
            for prefix in config.prefix() {
                writeln!(merged_writer, "{}", prefix)?;
                merged_len += 1;
            }
        }

        if files.len() == 1 {
            let file = File::open(files[0].clone()).with_context(|| format!("path: {}", files[0].to_string_lossy()))?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();

            while reader.read_line(&mut line)? > 0 {
                merged_writer.write(line.as_bytes())?;
                line = String::new();
                merged_len += 1;
            }
            std::fs::remove_file(files[0].clone())?;
        } else {
            let mut unmerged_files: BinaryHeap<UnmergedChunkFile> = files.into_iter()
                .map(
                    |path| UnmergedChunkFile::new(
                        path,
                        config.fields(),
                        config.field_separator(),
                        config.order().clone(),
                    )
                        .unwrap()
                )
                .collect();
            while unmerged_files.len() > 1 {
                let mut current_min = unmerged_files.pop().unwrap();
                let unmerged_min = unmerged_files.peek().unwrap();

                let mut current_min_done = false;
                // comparison operators are flipped to work with BinaryHeap (Max Heap)
                while &current_min >= unmerged_min {
                    let line_record = current_min.line_record();
                    if line_record.is_some() {
                        let line = line_record.unwrap().line();
                        merged_writer.write(line.as_bytes())?;
                        merged_len += 1;
                    } else {
                        current_min_done = true;
                        if remove_merged {
                            std::fs::remove_file(current_min.path())?;
                        }
                        break;
                    }
                }
                if !current_min_done {
                    unmerged_files.push(current_min)
                }
            }
            let mut current_min = unmerged_files.pop().unwrap();
            loop {
                let line_record = current_min.line_record();
                if line_record.is_some() {
                    let line = line_record.unwrap().line();
                    merged_writer.write(line.as_bytes())?;
                    merged_len += 1;
                } else {
                    std::fs::remove_file(current_min.path())?;
                    break;
                }
            }

            log::info!("Finished merging sorted files, thread: {}, merged length: {} lines", thread::current().name().unwrap_or("unnamed"), merged_len);
        }
        if add_prefix_suffix {
            for suffix in config.suffix() {
                writeln!(merged_writer, "{}", suffix)?;
                merged_len += 1;
            }
        }
        Ok((path, merged_len))
    }

    fn internal_sort(input_files: &Vec<PathBuf>, config: &Config, output: &PathBuf) -> Result<(), anyhow::Error> {
        log::info!("Start parallel sort");
        let mut thread_pool_builder = ThreadPoolBuilder::new();
        let mut sorting_pool = thread_pool_builder
            .with_name("sorting".to_string())
            .with_tasks(config.tasks())
            .with_queue_size(config.queue_size())
            .with_shutdown_mode(ShutdownMode::CompletePending)
            .build()
            .unwrap();

        sorting_pool.set_thread_local(&CONFIG, Some(config.clone()));

        for path in input_files {
            for chunk in ChunkIterator::new(path, config.chunk_size_bytes(), config.endl()).unwrap() {
                let sort_command = Box::new(SortCommand::new(Some(chunk)));
                sorting_pool.submit(sort_command);
            }
        }

        if config.concurrent_merge() {
            Self::merge_sorted_files(&sorting_pool);
        }

        let sorted_files = Self::collect_sorted_files(&mut sorting_pool);
        log::info!("Shutting down sorting pool");
        sorting_pool.shutdown();
        sorting_pool.join()?;

        let (path, _lines) = Self::internal_merge(sorted_files, &config, true, true)?;

        std::fs::rename(path.clone(), output.clone())
            .with_context(|| anyhow!("Rename {} to {}", path.to_string_lossy(), output.to_string_lossy()))?;
        log::info!("Finish parallel sort");
        Ok(())
    }
}
