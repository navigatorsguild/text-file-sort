use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::{anyhow, Context};

#[derive(Debug)]
pub(crate) struct Chunk {
    offset: u64,
    length: u64,
    path: PathBuf,
}

impl Chunk {
    pub(crate) fn new(offset: u64, length: u64, path: PathBuf) -> Chunk {
        Chunk {
            offset,
            length,
            path,
        }
    }

    pub(crate) fn offset(&self) -> u64 {
        self.offset
    }

    pub(crate) fn length(&self) -> u64 {
        self.length
    }

    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub(crate) struct ChunkIterator {
    path: PathBuf,
    reader: BufReader<File>,
    length: u64,
    reminder: u64,
    jump: u64,
    pos: u64,
    endl: char
}

impl ChunkIterator {
    pub(crate) fn new(path: &PathBuf, jump: u64, endl: char) -> Result<ChunkIterator, anyhow::Error> {
        let metadata = path.metadata()
            .with_context(|| anyhow!("path: {}", path.display()))?;
        let length = metadata.len();
        let reminder = length;
        let file = File::open(path)
            .with_context(|| anyhow!("path: {}", path.display()))?;

        Ok(
            ChunkIterator {
                path: path.clone(),
                reader: BufReader::new(file),
                length,
                reminder,
                jump,
                pos: 0,
                endl,
            }
        )
    }

    fn jump(&mut self) -> u64 {
        self.reader.seek(SeekFrom::Current(self.jump as i64))
            .unwrap_or_else(|_| panic!("Failed to jump. Path: {}, current position: {}, jump: {}",
                                       self.path.display(),
                                       self.pos,
                                       self.jump));
        let before_correction = self.reader.stream_position()
            .unwrap_or_else(|_| panic!("Failed to get position. Path: {}",
                                       self.path.display()));

        let mut line = Vec::new();
        self.reader.read_until(self.endl as u8, &mut line)
            .unwrap_or_else(|_| panic!("Failed to read. Path: {}, current position: {}",
                                       self.path.display(),
                                       before_correction));

        self.reader.stream_position()
            .unwrap_or_else(|_| panic!("Failed to get position. Path: {}",
                                       self.path.display()))
    }
}

impl Iterator for ChunkIterator {
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reminder == 0 {
            None
        } else if self.jump >= self.reminder {
            let chunk = Chunk::new(self.pos, self.reminder, self.path.clone());
            self.pos = self.length;
            self.reminder = 0;
            Some(chunk)
        } else {
            let current = self.jump();
            let actual_jump = current - self.pos;
            let chunk = Chunk::new(self.pos, actual_jump, self.path.clone());
            self.pos = current;
            self.reminder = self.length - current;
            Some(chunk)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
    use std::path::PathBuf;

    use crate::chunk_iterator::{Chunk, ChunkIterator};

    #[test]
    fn test_empty_file() -> Result<(), anyhow::Error> {
        let jump = 20000;
        let input_path = PathBuf::from("./tests/fixtures/empty-file.dat");
        let mut count = 0;
        let chunk_iterator = ChunkIterator::new(&input_path, jump, '\n')?;
        for _chunk in chunk_iterator {
            count += 1;
        }
        assert_eq!(count, 0);
        Ok(())
    }

    #[test]
    fn test_jump_greater_than_file() -> Result<(), anyhow::Error> {
        let input_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
        let jump = input_path.metadata().unwrap().len() + 18;
        let mut count = 0;
        let mut lines = 0;
        let chunk_iterator = ChunkIterator::new(&input_path, jump, '\n')?;
        for chunk in chunk_iterator {
            count += 1;
            assert_eq!(chunk.offset(), 0);
            assert_eq!(chunk.length(), input_path.metadata().unwrap().len());
            assert_eq!(chunk.path(), &input_path);
            lines += count_lines_in_chunk(&chunk).unwrap();
        }
        assert_eq!(count, 1);
        assert_eq!(lines, 10_000);
        Ok(())
    }

    #[test]
    fn test_jump_equal_to_file() -> Result<(), anyhow::Error> {
        let input_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
        let jump = input_path.metadata().unwrap().len() + 18;
        let mut count = 0;
        let mut lines = 0;
        let chunk_iterator = ChunkIterator::new(&input_path, jump, '\n')?;
        for chunk in chunk_iterator {
            assert_eq!(chunk.offset(), 0);
            assert_eq!(chunk.length(), input_path.metadata().unwrap().len());
            assert_eq!(chunk.path(), &input_path);
            count += 1;
            lines += count_lines_in_chunk(&chunk).unwrap();
        }
        assert_eq!(count, 1);
        assert_eq!(lines, 10_000);
        Ok(())
    }

    #[test]
    fn test_no_lines_lost() -> Result<(), anyhow::Error> {
        let input_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
        let jump = 10_000;
        let chunk_iterator = ChunkIterator::new(&input_path, jump, '\n')?;
        let mut lines = 0;
        for chunk in chunk_iterator {
            assert_eq!(chunk.path(), &input_path);
            lines += count_lines_in_chunk(&chunk).unwrap();
        }
        assert_eq!(lines, 10_000);
        Ok(())
    }

    fn count_lines_in_chunk(chunk: &Chunk) -> Result<usize, anyhow::Error> {
        let mut file = File::open(chunk.path())?;
        file.seek(SeekFrom::Start(chunk.offset))?;
        let mut buff = vec![0 as u8; chunk.length() as usize];
        file.read_exact(&mut buff)?;
        let reader = BufReader::new(buff.as_slice());
        let lines = reader.lines().count();
        Ok(lines)
    }
}