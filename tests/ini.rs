use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use renom::changes::{AppendIniEntry, Change, SetIniEntry};

#[test]
fn ini_append_should_not_strip_quotes() {
    let resources_dir = PathBuf::from("tests/resources");
    let original_config = resources_dir.join("ini/quoted_value.ini");
    let temp_dir = PathBuf::from("tests/temp");
    let staging_dir = temp_dir.join("ini/append_should_not_strip_quotes");
    let result_config = staging_dir.join("quoted_value.ini");
    if staging_dir.is_dir() {
        fs::remove_dir_all(&staging_dir).unwrap();
    }
    fs::create_dir_all(&staging_dir).unwrap();
    fs::copy(&original_config, &result_config).unwrap();

    let append_ini_entry = AppendIniEntry::new(&result_config, "test", "test", "test");
    let append_change = Change::AppendIniEntry(append_ini_entry);
    let _revert = append_change.apply(&staging_dir).unwrap();

    let actual = BufReader::new(File::open(result_config).unwrap())
        .lines()
        .next()
        .unwrap()
        .unwrap();
    let expected = r#"key="value""#;
    assert_eq!(actual, expected);
}

#[test]
fn ini_set_should_not_strip_quotes() {
    let resources_dir = PathBuf::from("tests/resources");
    let original_config = resources_dir.join("ini/quoted_value.ini");
    let temp_dir = PathBuf::from("tests/temp");
    let staging_dir = temp_dir.join("ini/set_should_not_strip_quotes");
    let result_config = staging_dir.join("quoted_value.ini");
    if staging_dir.is_dir() {
        fs::remove_dir_all(&staging_dir).unwrap();
    }
    fs::create_dir_all(&staging_dir).unwrap();
    fs::copy(&original_config, &result_config).unwrap();

    let set_ini_entry = SetIniEntry::new(&result_config, "test", "test", "test");
    let set_change = Change::SetIniEntry(set_ini_entry);
    let _revert = set_change.apply(&staging_dir).unwrap();

    let actual = BufReader::new(File::open(result_config).unwrap())
        .lines()
        .next()
        .unwrap()
        .unwrap();
    let expected = r#"key="value""#;
    assert_eq!(actual, expected);
}
