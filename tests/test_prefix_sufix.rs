use std::fs;
use std::path::PathBuf;
use text_file_sort::sort::Sort;

mod common;

#[test]
fn test_prefix_suffix_parallel() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-1000.dat");
    let output_path = common::temp_file_name("./target/parallel-results/");
    let tmp_path = PathBuf::from("./target/parallel-results/");

    let mut text_file_sort = Sort::new(vec![input_path.clone()], output_path.clone());
    text_file_sort.with_tasks(15);
    text_file_sort.add_prefix_line("first line".to_string());
    text_file_sort.add_prefix_line("second line".to_string());
    text_file_sort.add_suffix_line("penultimate line".to_string());
    text_file_sort.add_suffix_line("last line".to_string());
    text_file_sort.with_tmp_dir(tmp_path.clone());
    text_file_sort.sort()?;

    let lines = common::read_lines(output_path.clone())?;
    assert_eq!(lines[0], "first line".to_string());
    assert_eq!(lines[1], "second line".to_string());
    assert_eq!(lines[1002], "penultimate line".to_string());
    assert_eq!(lines[1003], "last line".to_string());
    fs::remove_file(output_path)?;
    Ok(())
}
