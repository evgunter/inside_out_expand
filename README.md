This crate exports two proc macros: `inside_out_expand` and `inside_out_expand_ignore_expansion_failure`.
These macros are used to wrap several layers of macro invocations, and alter the order in which they're expanded from outside-in to inside-out: they repeatedly expand macro invocations which contain no other macro invocations until all macro invocations have been expanded.
All expansion of macros is performed using the `proc_macro_expand` feature; this feature only allows expansion of macros which output a literal, so these macros have the same restriction. `inside_out_expand` will panic if it encounters a macro which does not have a literal as output; `inside_out_expand_ignore_expansion_failure` will skip expanding this macro instead.

## Example

```rust
macro_rules! expects_literal {
    ("a" $body:expr) => { $body };
}

macro_rules! produces_literal {
    ("b") => { "a" };
}

// This fails without inside_out_expand: expects_literal receives the
// unexpanded produces_literal!("b") tokens, which don't match "a".
// With inside_out_expand, produces_literal is expanded first.
let result = inside_out_expand!(expects_literal!(produces_literal!("b") "z"));
assert_eq!(result, "z");
```
