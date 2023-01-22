use std::path::Path;

pub fn expect_files_exist(dir: &Path, expected: &[&str]) {
    let mut actual: Vec<String> = dir
        .read_dir()
        .expect("Could not read directory")
        .into_iter()
        .map(|b| b.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    actual.sort();
    assert_eq!(actual, expected);
}
