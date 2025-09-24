use super::RenameScheme;
use test_case::test_case;

#[rustfmt::skip]
#[test_case("HelloWorld", "helloWorld" ; "camel_case_1")]
#[test_case("Hello_World", "helloWorld" ; "camel_case_2")]
#[test_case("hello_world", "helloWorld" ; "camel_case_3")]
#[test_case("__hello_world__", "helloWorld" ; "camel_case_4")]
#[test_case("HELLOWorld", "helloWorld" ; "camel_case_5")]
#[test_case("helloWORLD", "helloWorld" ; "camel_case_6")]
#[test_case("HELLO_WORLD", "helloWorld" ; "camel_case_7")]
#[test_case("hello_beautiful_world", "helloBeautifulWorld" ; "camel_case_8")]
#[test_case("helloBeautifulWorld", "helloBeautifulWorld" ; "camel_case_9")]
#[test_case("switch_to_term", "switchToTerm" ; "camel_case_10")]
#[test_case("_a_b_c_", "aBC" ; "camel_case_11")]
fn camel_case(input: &str, expected: &str) {
    assert_eq!(RenameScheme::camel_case(input).as_str(), expected);
}

#[rustfmt::skip]
#[test_case("HelloWorld", "HelloWorld" ; "pascal_case_1")]
#[test_case("Hello_World", "HelloWorld" ; "pascal_case_2")]
#[test_case("hello_world", "HelloWorld" ; "pascal_case_3")]
#[test_case("__hello_world__", "HelloWorld" ; "pascal_case_4")]
#[test_case("HELLOWorld", "HelloWorld" ; "pascal_case_5")]
#[test_case("helloWORLD", "HelloWorld" ; "pascal_case_6")]
#[test_case("HELLO_WORLD", "HelloWorld" ; "pascal_case_7")]
fn pascal_case(input: &str, expected: &str) {
    assert_eq!(RenameScheme::pascal_case(input).as_str(), expected);
}
