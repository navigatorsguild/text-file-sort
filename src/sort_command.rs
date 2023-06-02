use std::cell::RefCell;
use std::cmp::{max, Reverse};
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use anyhow::{anyhow, Context};
use command_executor::command::Command;

use crate::chunk_iterator::Chunk;
use crate::config::Config;
use crate::line_record::LineRecord;
use crate::sort::{create_tmp_file, get_line_capacity, get_line_records_capacity, get_tl_config, set_line_capacity, set_line_records_capacity, Sort, SORTED_FILES};
use crate::sorted_chunk_file::SortedChunkFile;

pub(crate) struct SortCommand {
    chunk: Option<Chunk>,
}

impl SortCommand {
    pub(crate) fn new(chunk: Option<Chunk>) -> SortCommand {
        SortCommand {
            chunk,
        }
    }

    fn write_sorted_chunk(sorted_files: &RefCell<BinaryHeap<Reverse<SortedChunkFile>>>, chunk: Vec<LineRecord>, chunk_size: usize, config: &Config) {
        let tmp_file = create_tmp_file(config);
        let (chunk_file, path) = tmp_file
            .keep()
            .or_else(|e| Err(anyhow!("Failed to persist temp file: {}", e.to_string())))
            .unwrap();

        let mut buf_writer = BufWriter::new(chunk_file);

        for line_record in chunk {
            buf_writer.write(line_record.line().as_bytes()).unwrap();
        }

        sorted_files
            .borrow_mut()
            .push(Reverse(SortedChunkFile::new(path, chunk_size)));
    }

    fn read_records(&self) -> Result<Vec<LineRecord>, anyhow::Error> {
        let line_records_capacity = get_line_records_capacity();
        let mut line_capacity = get_line_capacity();
        let mut line_records = Vec::with_capacity(line_records_capacity);
        match &self.chunk {
            None => {}
            Some(file_chunk) => {
                let mut file = File::open(file_chunk.path())?;
                file.seek(SeekFrom::Start(file_chunk.offset()))?;
                let mut buff = vec![0 as u8; file_chunk.length() as usize];
                file.read_exact(&mut buff)?;
                let mut reader = BufReader::new(buff.as_slice());
                let config = get_tl_config();

                let mut n = 0;
                let mut line = String::with_capacity(line_capacity);
                while reader.read_line(&mut line)? != 0 {
                    n += 1;
                    if config.ignore_empty() && line.trim().is_empty() {
                        line.clear();
                        continue;
                    }

                    if let Some(r) = config.ignore_lines() {
                        if r.is_match(line.trim()) {
                            line.clear();
                            continue;
                        }
                    }
                    line_capacity = max(line.len(), line_capacity);
                    let line_record = LineRecord::new(
                        line,
                        config.fields(),
                        config.field_separator(),
                        config.order().clone(),
                    )
                        .with_context(||
                            format!(
                                "file: {}, chunk offset: {}, line within chunk: {}",
                                file_chunk.path().to_string_lossy(),
                                file_chunk.offset(),
                                n
                            )
                        )?;
                    line_records.push(line_record);
                    line = String::with_capacity(line_capacity);
                }
            }
        }
        set_line_capacity(line_capacity);
        set_line_records_capacity(max(line_records.len(), line_records_capacity));
        Ok(line_records)
    }
}

impl Command for SortCommand {
    fn execute(&self) -> Result<(), anyhow::Error> {
        let config = get_tl_config();
        let mut chunk = self.read_records()?;
        chunk.sort();
        SORTED_FILES.with(
            |sorted_files| {
                let chunk_size = chunk.len();

                if sorted_files.borrow().len() < config.files() / config.tasks() {
                    Self::write_sorted_chunk(sorted_files, chunk, chunk_size, &config);
                } else {
                    let f1 = sorted_files.borrow_mut().pop().unwrap().0;
                    let f2 = sorted_files.borrow_mut().pop().unwrap().0;
                    let mut files = Vec::new();
                    files.push(f1.path().clone());
                    files.push(f2.path().clone());

                    let (path, lines) = Sort::internal_merge(files, &config, true, false).unwrap();
                    let merged = SortedChunkFile::new(path, lines);
                    sorted_files.borrow_mut().push(Reverse(merged));
                    Self::write_sorted_chunk(sorted_files, chunk, chunk_size, &config);
                }
                Ok::<(), anyhow::Error>(())
            }
        )?;
        Ok(())
    }
}
