use lisan::run;

const EX1: &str = include_str!("../examples/announcement/ex1_beginner.lsn");
const EX2: &str = include_str!("../examples/announcement/ex2_lists_functions.lsn");
const EX3: &str = include_str!("../examples/announcement/ex3_arduino.lsn");

#[test]
fn announcement_example_1_beginner() {
    let out = run(EX1).expect("example 1 must run");
    println!("EX1 OUTPUT: {:?}", out);
    assert_eq!(out.len(), 2);
}

#[test]
fn announcement_example_2_lists_functions() {
    let out = run(EX2).expect("example 2 must run");
    println!("EX2 OUTPUT: {:?}", out);
    assert!(out.iter().any(|l| l.contains("المجموع")));
}

#[test]
fn announcement_example_3_arduino() {
    let out = run(EX3).expect("example 3 must run");
    println!("EX3 OUTPUT: {:?}", out);
    assert!(out.iter().any(|l| l.contains("مرحبا من لسان") || l.contains("السطر الأول")));
}
