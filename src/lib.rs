#![feature(extend_one)]
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

fn inside_out_expand_inner(input: TokenStream, ignore_failed_macro_expansion: bool) -> TokenStream {
    // takes macro invocations in the input and expands the most deeply nested macro invocations first
    // (with ties being broken by expanding the leftmost macro invocation first)
    let mut current_tokens: TokenStream2 = input.into();
    loop {
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
                                let current_macro_ident = match current_pass_new.pop() {
                                    Some(TokenTree2::Ident(ident)) => ident,
                                    _ => panic!("Expected an identifier at the end of current_pass_new")
                                };
                                let current_invocation: TokenStream = TokenStream2::from_iter(vec![TokenTree2::Ident(current_macro_ident), TokenTree2::Punct(punct), TokenTree2::Group(Group::new(group.delimiter(), inner_expanded))]).into();
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
                        current_pass_new.extend_one(TokenTree2::Punct(punct));
                    }
                }
                v => current_pass_new.extend_one(v),
            }
        }
        if !expansion_performed {
            return current_pass_new.into_iter().collect::<TokenStream2>().into();
        }
        current_tokens = current_pass_new.into_iter().collect::<TokenStream2>();
    }
}
