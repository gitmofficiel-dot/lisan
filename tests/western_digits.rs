use lisan::run;

#[test]
fn western_digits_all_ten() {
    let src = "اطبع(0)\nاطبع(1)\nاطبع(2)\nاطبع(3)\nاطبع(4)\nاطبع(5)\nاطبع(6)\nاطبع(7)\nاطبع(8)\nاطبع(9)\n";
    assert_eq!(
        run(src).unwrap(),
        vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    );
}

#[test]
fn western_digits_multi_digit_number() {
    assert_eq!(run("اطبع(0123456789)").unwrap(), vec!["123456789"]);
}

#[test]
fn western_digits_arithmetic() {
    let src = "دع أ = 10\nدع ب = 20\nدع ج = 3\nاطبع(أ + ب * ج)\n";
    assert_eq!(run(src).unwrap(), vec!["70"]);
}

#[test]
fn western_digits_scientific_notation() {
    assert_eq!(run("اطبع(2e3)").unwrap(), vec!["2000"]);
}

#[test]
fn western_digits_in_identifiers() {
    let src = "دع متغير1 = 7\nدع متغير2 = 8\nاطبع(متغير1 + متغير2)\n";
    assert_eq!(run(src).unwrap(), vec!["15"]);
}

#[test]
fn all_three_digit_systems_give_the_same_value() {
    let src = "اطبع(42)\nاطبع(٤٢)\nاطبع(۴۲)\n";
    assert_eq!(run(src).unwrap(), vec!["42", "42", "42"]);
}
