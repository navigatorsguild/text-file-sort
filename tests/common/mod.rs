use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::io::{BufRead, BufReader};
use std::fs::File;
use data_encoding::HEXLOWER;

pub fn setup() {
    let results_dir_path = PathBuf::from_str("./target/results/").unwrap();
    let parallel_results_dir_path = PathBuf::from_str("./target/parallel-results/").unwrap();

    if !results_dir_path.exists() {
        fs::create_dir_all(&results_dir_path).unwrap_or_else(|_|
            panic!("Failed to create results directory: {:?}", results_dir_path)
        );
    } else {
        println!("Results directory exists at {:?}", results_dir_path);
    }

    if !parallel_results_dir_path.exists() {
        fs::create_dir_all(&parallel_results_dir_path).unwrap_or_else(|_|
            panic!("Failed to create parallel results directory: {:?}", parallel_results_dir_path)
        );
    } else {
        println!("Results directory exists at {:?}", parallel_results_dir_path);
    }
}

#[allow(dead_code)]
pub fn read_lines(path: PathBuf) -> Result<Vec<String>, anyhow::Error> {
    let reader = BufReader::new(File::open(path)?);
    let lines = reader.lines().map(|x| x.unwrap()).collect();
    Ok(lines)
}

#[allow(dead_code)]
pub fn temp_file_name(dir: &str) -> PathBuf {
    let mut result = PathBuf::from(dir);
    let name = HEXLOWER.encode(&rand::random::<[u8; 16]>());
    result.push(name);
    result
}
