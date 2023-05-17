//! This crate implements a sort algorithm for text files composed of lines or line records. For example
//! CSV or TSV.
//!
//! A data file composed of lines or line records, that is lines that are composed of fields separated
//! by a delimiter, can be sorted using this crate. Example for such files are
//! [pg_dump](https://www.postgresql.org/docs/current/app-pgdump.html),
//! [CSV](https://www.rfc-editor.org/rfc/rfc4180) and [GTFS](https://gtfs.org/schedule/reference/) data files.
//! The motivation for writing this module was the need to sort pg_dump files of the [OpenStreetMap](https://www.openstreetmap.org/)
//! database containing billions of lines by the primary key of each table before converting the data
//! to PBF format.
//!
//! This implementation can be used to sort very large files, taking advantage of multiple CPU
//! cores and providing memory usage control.
//!
//! # Examples
//! ```
//! use std::path::PathBuf;
//! use text_file_sort::sort::Sort;
//!
//! // optimized for use with Jemalloc
//! use tikv_jemallocator::Jemalloc;
//! #[global_allocator]
//! static GLOBAL: Jemalloc = Jemalloc;
//!
//! // parallel record sort
//! fn sort_records(input: PathBuf, output: PathBuf, tmp: PathBuf) -> Result<(), anyhow::Error> {
//!    let mut text_file_sort = Sort::new(vec![input.clone()], output.clone());
//!
//!     // set number of CPU cores the sort will attempt to use. When given the number that exceeds
//!     // the number of available CPU cores the work will be split among available cores with
//!     // somewhat degraded performance. The default is to use all available cores.
//!     text_file_sort.with_tasks(2);
//!
//!     // set the directory for intermediate results. The default is the system temp dir -
//!     // std::env::temp_dir(), however, for large files it is recommended to provide a dedicated
//!     // directory for intermediate files, preferably on the same file system as the output result.
//!     text_file_sort.with_tmp_dir(tmp);
//!
//!     text_file_sort.sort()
//! }
//! ```
//!

pub(crate) mod sort_command;
pub(crate) mod line_record;
pub(crate) mod key;
pub(crate) mod sorted_chunk_file;
pub(crate) mod unmerged_chunk_file;
pub(crate) mod config;
pub(crate) mod chunk_iterator;

pub mod sort;
pub mod field;
pub mod field_type;
pub mod order;
