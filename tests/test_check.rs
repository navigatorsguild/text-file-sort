use std::fs;
use std::path::PathBuf;
use text_file_sort::field::Field;
use text_file_sort::field_type::FieldType;
use text_file_sort::order::Order;
use text_file_sort::sort::Sort;


mod common;

#[test]
fn test_check_sorted() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-1000.dat");

    let text_file_sort = Sort::new(vec![input_path.clone(), input_path.clone()], PathBuf::new());
    let result = text_file_sort.check()?;
    assert_eq!(result, true);
    Ok(())
}

#[test]
fn test_check_sorted_desc() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-desc-1000.dat");

    let mut text_file_sort = Sort::new(vec![input_path.clone(), input_path.clone()], PathBuf::new());
    text_file_sort.with_order(Order::Desc);
    let result = text_file_sort.check()?;
    assert_eq!(result, true);
    Ok(())
}

#[test]
fn test_check_not_sorted() -> Result<(), anyhow::Error> {
    common::setup();
    let input_path = PathBuf::from("./tests/fixtures/sorted-1000.dat");
    let random_path = common::temp_file_name("./target/results/");

    let mut random_sort = Sort::new(vec![input_path.clone()], random_path.clone());
    random_sort.add_field(Field::new(0, FieldType::String).with_random(true));
    random_sort.sort()?;


    let text_file_sort = Sort::new(vec![random_path.clone()], PathBuf::new());
    let result = text_file_sort.check()?;
    assert_eq!(result, false);
    fs::remove_file(random_path)?;
    Ok(())
}