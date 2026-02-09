#![feature(proc_macro_expand)]

use proc_macro::TokenStream;
use proc_macro2::{Group, TokenTree as TokenTree2, TokenStream as TokenStream2};

#[proc_macro]
pub fn inside_out_expand(input: TokenStream) -> TokenStream {
    inside_out_expand_inner(input, false)
}

#[proc_macro]
pub fn inside_out_expand_ignore_expansion_failure(input: TokenStream) -> TokenStream {
    inside_out_expand_inner(input, true)
}

/// Pops a macro path from the end of the token vector.
/// Handles simple identifiers (e.g., `stringify`) and path-qualified names
/// (e.g., `std::stringify`, `::std::stringify`).
fn pop_macro_path(tokens: &mut Vec<TokenTree2>) -> Vec<TokenTree2> {
    let macro_ident = match tokens.pop() {
        Some(TokenTree2::Ident(ident)) => ident,
        _ => panic!("Expected an identifier before '!' in macro invocation")
    };

    // Collect path tokens in reverse order, then reverse at the end
    let mut path_tokens: Vec<TokenTree2> = vec![TokenTree2::Ident(macro_ident)];

    loop {
        let len = tokens.len();
        if len < 2 {
            break;
        }
        // Check if the last two tokens form `::`
        let is_path_sep = matches!(
            (&tokens[len - 2], &tokens[len - 1]),
            (TokenTree2::Punct(p1), TokenTree2::Punct(p2))
            if p1.as_char() == ':' && p2.as_char() == ':'
        );
        if !is_path_sep {
            break;
        }
        // Pop the `::`
        path_tokens.push(tokens.pop().unwrap());
        path_tokens.push(tokens.pop().unwrap());

        // Check if there's a preceding Ident (path segment)
        match tokens.last() {
            Some(TokenTree2::Ident(_)) => {
                path_tokens.push(tokens.pop().unwrap());
            }
            _ => {
                // Leading `::` with no preceding ident; stop
                break;
            }
        }
    }

    path_tokens.reverse();
    path_tokens
}

/// Maximum number of expansion passes, mirroring the compiler's default recursion_limit of 128.
const EXPANSION_LIMIT: usize = 128;

fn inside_out_expand_inner(input: TokenStream, ignore_failed_macro_expansion: bool) -> TokenStream {
    // takes macro invocations in the input and expands the most deeply nested macro invocations first
    // (with ties being broken by expanding the leftmost macro invocation first)
    let mut current_tokens: TokenStream2 = input.into();
    for _ in 0..EXPANSION_LIMIT {
        let mut expansion_performed = false;
        let mut current_pass_new: Vec<TokenTree2> = Vec::new();
        let mut current_pass_remaining = current_tokens.into_iter();
        'single_pass: loop {
            let token = match current_pass_remaining.next() {
                Some(token) => token,
                None => break 'single_pass,  // we've finished iterating over current_tokens
            };
            // match this token to determine whether it is a macro invocation
            match token {
                TokenTree2::Punct(punct) => {
                    if punct.as_char() == '!' {
                        // this is a macro invocation, so the next token should be a group
                        match current_pass_remaining.next() {
                            Some(TokenTree2::Group(group)) => {
                                // recursively expand any macro invocations in the group
                                let inner_expanded = inside_out_expand(group.stream().into()).into();

                                // then expand the current macro invocation
                                let mut macro_path = pop_macro_path(&mut current_pass_new);
                                macro_path.push(TokenTree2::Punct(punct));
                                macro_path.push(TokenTree2::Group(Group::new(group.delimiter(), inner_expanded)));
                                let current_invocation: TokenStream = TokenStream2::from_iter(macro_path).into();
                                let current_expanded: TokenStream2 = match current_invocation.expand_expr() {
                                    Ok(expanded) => {
                                        expansion_performed = true;
                                        expanded
                                    },
                                    Err(e) => if ignore_failed_macro_expansion {
                                        // in this case, leave the macro invocation unexpanded. since expand_expr only supports expansion into a literal, this can be convenient if multiple kinds of macros are being used.
                                        current_invocation
                                    } else {
                                        panic!("Error expanding macro invocation: {}", e)
                                    }
                                }.into();
                                current_pass_new.extend(current_expanded);
                            }
                            _ => panic!("Expected a group after a '!' in the input")
                        }
                    } else {
                        current_pass_new.push(TokenTree2::Punct(punct));
                    }
                }
                v => current_pass_new.push(v),
            }
        }
        if !expansion_performed {
            return current_pass_new.into_iter().collect::<TokenStream2>().into();
        }
        current_tokens = current_pass_new.into_iter().collect::<TokenStream2>();
    }
    panic!("inside_out_expand: expansion limit of {} reached, possible infinite macro expansion", EXPANSION_LIMIT);
}
