use lisan::run;

#[test]
fn negative_index_list() {
    assert_eq!(run("اطبع([1، 2، 3][-1])؛").unwrap(), ["3"]);
    assert_eq!(run("اطبع([1، 2، 3][-3])؛").unwrap(), ["1"]);
}

#[test]
fn negative_index_text() {
    assert_eq!(run("اطبع(\"abc\"[-1])؛").unwrap(), ["c"]);
}

#[test]
fn negative_index_out_of_range_errors() {
    assert!(run("اطبع([1، 2][-3])؛").is_err());
}
