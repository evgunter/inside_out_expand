#![cfg(test)]

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
    // Verify that nested macros fail to compile without inside_out_expand,
    // since the outer macro receives the unexpanded inner invocation.
    let t = trybuild::TestCases::new();
    t.compile_fail("compile_fail/double_depth_original.rs");
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
    let t = trybuild::TestCases::new();
    t.compile_fail("compile_fail/dont_ignore_failed_expansion.rs");
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
