#![cfg(test)]

use trybuild::TestCases;
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;
use inside_out_expand::{inside_out_expand, inside_out_expand_ignore_expansion_failure};

macro_rules! macro_a_to_end {
    ("a" $body:expr) => {
        $body
    };
}

macro_rules! macro_b_to_a {
    ("b" $body:expr) => {
        "a"
    };
}

macro_rules! macro_c_to_b {
    ("c" $body:expr) => {
        "b"
    };
}

#[test]
fn single_depth() {
    assert_eq!(inside_out_expand!(macro_a_to_end!("a" "q")), "q");
}

#[test]
fn double_depth_original() {
    // make a temporary file to check that "macro_a_to_end!(macro_b_to_a!(b "q"))" does not compile
    let tmp_dir = TempDir::new().unwrap();
    let tmp_file_path = tmp_dir.path().join("test_double_depth_compile.rs");
    let mut tmp_file = File::create(tmp_file_path.clone()).unwrap();
    // the innermost macro would need to be expanded first for this to compile
    writeln!(tmp_file, r#"
macro_rules! macro_a_to_end {{
    ("a" $body:expr) => {{
        $body
    }};
}}

macro_rules! macro_b_to_a {{
    ("b" $body:expr) => {{
        "a"
    }};
}}


fn main() {{
    let _ = macro_a_to_end!(macro_b_to_a!("b" "q") "z");
}}
"#).unwrap();

    let tmp_answer_file_path = tmp_dir.path().join("test_double_depth_compile.stderr");
    let mut tmp_answer_file = File::create(tmp_answer_file_path.clone()).unwrap();
    writeln!(tmp_answer_file, r#"error: no rules expected `macro_b_to_a`
 --> {}:16:29
  |
 2 | macro_rules! macro_a_to_end {{
   | --------------------------- when calling this macro
...
16 |     let _ = macro_a_to_end!(macro_b_to_a!("b" "q") "z");
   |                             ^^^^^^^^^^^^ no rules expected this token in macro call
   |
note: while trying to match `"a"`
  --> {}:3:6
   |
 3 |     ("a" $body:expr) => {{
   |      ^^^"#, tmp_file_path.display(), tmp_file_path.display()).unwrap();

    {
        // this has to go out of scope before the temporary file does, or the file may be deleted before it can read it
        let t = TestCases::new();
        t.compile_fail(tmp_file_path.clone());
    }
}

#[test]
fn double_depth_reversed() {
    assert_eq!(inside_out_expand!(macro_a_to_end!(macro_b_to_a!("b" "q") "z")), "z");
}

macro_rules! macro_nonlit_out {
    ($body:expr) => {
        {
            const DEFINED_IN_MACRO: &str = $body;
            DEFINED_IN_MACRO
        }
    };
}

#[test]
fn test_ignore_failed_macro_expansion() {
    // macro_nonlit_out doesn't emit a literal, so it causes an internal error in the expansion macro;
    // this tests that such an error is ignored
    let d = inside_out_expand_ignore_expansion_failure!(macro_nonlit_out!(macro_a_to_end!("a" "q")));
    assert_eq!(d, "q");
}

#[test]
fn test_dont_ignore_failed_macro_expansion() {
    // macro_nonlit_out doesn't emit a literal, so it causes an internal error in the expansion macro;
    // this tests that such an error does indeed occur. (this is mostly relevant to demonstrate the soundness of the test above.)
    let tmp_dir = TempDir::new().unwrap();
    let tmp_file_path = tmp_dir.path().join("test_ignore_failed_macro_expansion.rs");
    let mut tmp_file = File::create(tmp_file_path.clone()).unwrap();
    writeln!(tmp_file, r#"
use inside_out_expand::inside_out_expand;

macro_rules! macro_a_to_end {{
    ("a" $body:expr) => {{
        $body
    }};
}}

macro_rules! macro_nonlit_out {{
    ($body:expr) => {{
        {{
            const DEFINED_IN_MACRO: &str = $body;
            DEFINED_IN_MACRO
        }}
    }};
}}


fn main() {{
    let _ = inside_out_expand!(macro_nonlit_out!(macro_a_to_end!("a" "q")));
}}
"#).unwrap();
    let tmp_answer_file_path = tmp_dir.path().join("test_ignore_failed_macro_expansion.stderr");
    let mut tmp_answer_file = File::create(tmp_answer_file_path.clone()).unwrap();
    writeln!(tmp_answer_file, r#"error: proc macro panicked
  --> {}:21:13
   |
21 |     let _ = inside_out_expand!(macro_nonlit_out!(macro_a_to_end!("a" "q")));
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: Error expanding macro invocation: macro expansion failed"#, tmp_file_path.display()).unwrap();

    {
        // this has to go out of scope before the temporary file does, or the file may be deleted before it can read it
        let t = TestCases::new();
        t.compile_fail(tmp_file_path.clone());
    }
}

#[test]
fn empty_input() {
    // inside_out_expand with no macro invocations should pass through unchanged
    let result = inside_out_expand!("hello");
    assert_eq!(result, "hello");
}

#[test]
fn triple_depth() {
    // three levels of nesting: macro_c_to_b expands to "b", then macro_b_to_a expands to "a", then macro_a_to_end returns "z"
    assert_eq!(
        inside_out_expand!(macro_a_to_end!(macro_b_to_a!(macro_c_to_b!("c" "q") "q") "z")),
        "z"
    );
}

#[test]
fn bracket_delimiters() {
    // macros invoked with square brackets should also work
    assert_eq!(inside_out_expand!(macro_a_to_end!["a" "q"]), "q");
}
