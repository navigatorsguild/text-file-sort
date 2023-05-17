use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use text_file_sort::field::Field;
use text_file_sort::field_type::FieldType;
use text_file_sort::order::Order;
use text_file_sort::sort::Sort;

mod common;

#[test]
fn test_parallel_sort() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
    let random_path = common::temp_file_name("./target/parallel-results/");
    let output_path = common::temp_file_name("./target/parallel-results/");
    let tmp_path = PathBuf::from("./target/parallel-results/");
    let mut random_sort = Sort::new(vec![input_path.clone()], random_path.clone());
    random_sort.add_field(Field::new(0, FieldType::String).with_random(true));
    random_sort.sort()?;

    let mut text_file_sort = Sort::new(vec![random_path.clone()], output_path.clone());
    text_file_sort.with_tasks(15);
    text_file_sort.with_tmp_dir(tmp_path.clone());
    text_file_sort.sort()?;

    let mut input = String::new();
    let mut output = String::new();
    BufReader::new(File::open(input_path.clone())?).read_to_string(&mut input)?;
    BufReader::new(File::open(output_path.clone())?).read_to_string(&mut output)?;
    assert_eq!(input, output);
    fs::remove_file(random_path)?;
    fs::remove_file(output_path)?;
    Ok(())
}

#[test]
fn test_parallel_sort_desc() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-10000.dat");
    let random_path = common::temp_file_name("./target/parallel-results/");
    let asc_output_path = common::temp_file_name("./target/parallel-results/");
    let desc_output_path = common::temp_file_name("./target/parallel-results/");
    let tmp_path = PathBuf::from("./target/parallel-results/");

    let mut random_sort = Sort::new(vec![input_path.clone()], random_path.clone());
    random_sort.add_field(Field::new(0, FieldType::String).with_random(true));
    random_sort.sort()?;

    let mut asc_sort = Sort::new(vec![random_path.clone()], asc_output_path.clone());
    asc_sort.with_tasks(2);
    asc_sort.with_tmp_dir(tmp_path.clone());
    asc_sort.sort()?;

    let mut desc_sort = Sort::new(vec![random_path.clone()], desc_output_path.clone());
    desc_sort.with_tasks(2);
    desc_sort.with_tmp_dir(tmp_path.clone());
    desc_sort.with_order(Order::Desc);
    desc_sort.sort()?;


    let asc_lines = common::read_lines(asc_output_path.clone())?;
    let desc_lines = common::read_lines(desc_output_path.clone())?;
    let first: usize = 0;
    let last = asc_lines.len() - 1;
    assert_eq!(asc_lines[first], desc_lines[last]);
    assert_eq!(asc_lines[last], desc_lines[first]);
    fs::remove_file(random_path)?;
    fs::remove_file(asc_output_path)?;
    fs::remove_file(desc_output_path)?;
    Ok(())
}
