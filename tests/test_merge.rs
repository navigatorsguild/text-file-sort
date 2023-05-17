use std::fs;
use std::path::PathBuf;
use text_file_sort::sort::Sort;

mod common;

#[test]
fn test_merge() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-1000.dat");
    let output_path = common::temp_file_name("./target/results/");

    let mut input_files = Vec::new();
    for i in 0..10 {
        let mut path = output_path.clone();
        path.set_file_name(format!("sorted-1000"));
        path.set_extension(format!("{i}"));
        fs::copy(input_path.clone(), path.clone())?;
        input_files.push(path.clone());
    }
    let text_file_sort = Sort::new(input_files, output_path.clone());
    text_file_sort.merge()?;

    let lines = common::read_lines(output_path.clone())?;
    assert_eq!(lines.len(), 10000);
    fs::remove_file(output_path)?;
    Ok(())
}
