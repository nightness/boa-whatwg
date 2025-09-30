use std::path::PathBuf;

use test_case::test_case;

use super::*;

// Tests on Windows and Linux are different because of the path separator and the definition
// of absolute paths.
#[rustfmt::skip]
#[cfg(target_family = "unix")]
#[test_case(Some("/hello/ref.js"),      "a.js",             Ok("/base/a.js"))]
#[test_case(Some("/base/ref.js"),       "./b.js",           Ok("/base/b.js"))]
#[test_case(Some("/base/other/ref.js"), "./c.js",           Ok("/base/other/c.js"))]
#[test_case(Some("/base/other/ref.js"), "../d.js",          Ok("/base/d.js"))]
#[test_case(Some("/base/ref.js"),        "e.js",            Ok("/base/e.js"))]
#[test_case(Some("/base/ref.js"),        "./f.js",          Ok("/base/f.js"))]
#[test_case(Some("./ref.js"),           "./g.js",           Ok("/base/g.js"))]
#[test_case(Some("./other/ref.js"),     "./other/h.js",     Ok("/base/other/other/h.js"))]
#[test_case(Some("./other/ref.js"),     "./other/../h1.js", Ok("/base/other/h1.js"))]
#[test_case(Some("./other/ref.js"),     "./../h2.js",       Ok("/base/h2.js"))]
#[test_case(None,                       "./i.js",           Err(()))]
#[test_case(None,                       "j.js",             Ok("/base/j.js"))]
#[test_case(None,                       "other/k.js",       Ok("/base/other/k.js"))]
#[test_case(None,                       "other/../../l.js", Err(()))]
#[test_case(Some("/base/ref.js"),       "other/../../m.js", Err(()))]
#[test_case(None,                       "../n.js",          Err(()))]
fn resolve_test(ref_path: Option<&str>, spec: &str, expected: Result<&str, ()>) {
    let base = PathBuf::from("/base");

    let mut context = Context::default();
    let spec = js_string!(spec);
    let ref_path = ref_path.map(PathBuf::from);

    let actual = resolve_module_specifier(
        Some(&base),
        &spec,
        ref_path.as_deref(),
        &mut context,
    );
    assert_eq!(actual.map_err(|_| ()), expected.map(PathBuf::from));
}

// This tests the same cases as the previous test, but without a base path.
#[rustfmt::skip]
#[cfg(target_family = "unix")]
#[test_case(Some("hello/ref.js"),       "a.js",             Ok("a.js"))]
#[test_case(Some("base/ref.js"),        "./b.js",           Ok("base/b.js"))]
#[test_case(Some("base/other/ref.js"),  "./c.js",           Ok("base/other/c.js"))]
#[test_case(Some("base/other/ref.js"),  "../d.js",          Ok("base/d.js"))]
#[test_case(Some("base/ref.js"),        "e.js",             Ok("e.js"))]
#[test_case(Some("base/ref.js"),        "./f.js",           Ok("base/f.js"))]
#[test_case(Some("./ref.js"),           "./g.js",           Ok("g.js"))]
#[test_case(Some("./other/ref.js"),     "./other/h.js",     Ok("other/other/h.js"))]
#[test_case(Some("./other/ref.js"),     "./other/../h1.js", Ok("other/h1.js"))]
#[test_case(Some("./other/ref.js"),     "./../h2.js",       Ok("h2.js"))]
#[test_case(None,                       "./i.js",           Err(()))]
#[test_case(None,                       "j.js",             Ok("j.js"))]
#[test_case(None,                       "other/k.js",       Ok("other/k.js"))]
#[test_case(None,                       "other/../../l.js", Err(()))]
#[test_case(Some("/base/ref.js"),       "other/../../m.js", Err(()))]
#[test_case(None,                       "../n.js",          Err(()))]
fn resolve_test_no_base(ref_path: Option<&str>, spec: &str, expected: Result<&str, ()>) {
    let mut context = Context::default();
    let spec = js_string!(spec);
    let ref_path = ref_path.map(PathBuf::from);

    let actual = resolve_module_specifier(
        None,
        &spec,
        ref_path.as_deref(),
        &mut context,
    );
    assert_eq!(actual.map_err(|_| ()), expected.map(PathBuf::from));
}

#[rustfmt::skip]
#[cfg(target_family = "windows")]
#[test_case(Some("a:\\hello\\ref.js"),       "a.js",                Ok("a:\\base\\a.js"))]
#[test_case(Some("a:\\base\\ref.js"),        "./b.js",              Ok("a:\\base\\b.js"))]
#[test_case(Some("a:\\base\\other\\ref.js"), "./c.js",              Ok("a:\\base\\other\\c.js"))]
#[test_case(Some("a:\\base\\other\\ref.js"), "../d.js",             Ok("a:\\base\\d.js"))]
#[test_case(Some("a:\\base\\ref.js"),        "e.js",                Ok("a:\\base\\e.js"))]
#[test_case(Some("a:\\base\\ref.js"),        "./f.js",              Ok("a:\\base\\f.js"))]
#[test_case(Some(".\\ref.js"),               "./g.js",              Ok("a:\\base\\g.js"))]
#[test_case(Some(".\\other\\ref.js"),        "./other/h.js",        Ok("a:\\base\\other\\other\\h.js"))]
#[test_case(Some(".\\other\\ref.js"),        "./other/../h1.js",    Ok("a:\\base\\other\\h1.js"))]
#[test_case(Some(".\\other\\ref.js"),        "./../h2.js",          Ok("a:\\base\\h2.js"))]
#[test_case(None,                            "./i.js",              Err(()))]
#[test_case(None,                            "j.js",                Ok("a:\\base\\j.js"))]
#[test_case(None,                            "other/k.js",          Ok("a:\\base\\other\\k.js"))]
#[test_case(None,                            "other/../../l.js",    Err(()))]
#[test_case(Some("\\base\\ref.js"),          "other/../../m.js",    Err(()))]
#[test_case(None,                            "../n.js",             Err(()))]
fn resolve_test(ref_path: Option<&str>, spec: &str, expected: Result<&str, ()>) {
    let base = PathBuf::from("a:\\base");

    let mut context = Context::default();
    let spec = js_string!(spec);
    let ref_path = ref_path.map(PathBuf::from);

    let actual = resolve_module_specifier(
        Some(&base),
        &spec,
        ref_path.as_deref(),
        &mut context,
    );
    assert_eq!(actual.map_err(|_| ()), expected.map(PathBuf::from));
}
