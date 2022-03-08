extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

#[proc_macro_attribute]
pub fn before_each(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input: Item = match parse_macro_input!(input as Item) {
        Item::Mod(mut m) => {
            let (brace, items) = m.content.unwrap();
            let (before_each_fn, everything_else): (Vec<Item>, Vec<Item>) =
                items.into_iter().partition(|t| match t {
                    Item::Fn(f) => f.sig.ident == "before_each",
                    _ => false,
                });
            let before_each_fn_block = if before_each_fn.len() != 1 {
                panic!("The `before_each` macro attribute requires a single function named `before_each` in the body of the module it is called on.")
            } else {
                match before_each_fn.into_iter().next().unwrap() {
                    Item::Fn(f) => f.block,
                    _ => unreachable!(),
                }
            };

            let e: Vec<Item> = everything_else
                .into_iter()
                .map(|t| match t {
                    Item::Fn(mut f) => {
                        if f.attrs.iter().any(|attr| {
                            attr.path
                                .segments
                                .iter()
                                // only apply before_each biz to functions with test attributes, this
                                // includes variants like `tokio::test`, `test_case`
                                .any(|segment| segment.ident.to_string().contains("test"))
                        }) {
                            let mut b = before_each_fn_block.stmts.clone();
                            b.append(&mut f.block.stmts);
                            f.block.stmts = b;
                            Item::Fn(f)
                        } else {
                            Item::Fn(f)
                        }
                    }
                    e => e,
                })
                .collect();
            m.content = Some((brace, e));
            Item::Mod(m)
        }

        _ => {
            panic!("The `before_each` macro attribute is only valid when called on a module.")
        }
    };
    TokenStream::from(quote! {#input})
}

#[proc_macro_attribute]
pub fn after_each(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input: Item = match parse_macro_input!(input as Item) {
        Item::Mod(mut m) => {
            let (brace, items) = m.content.unwrap();
            let (after_each_fn, everything_else): (Vec<Item>, Vec<Item>) =
                items.into_iter().partition(|t| match t {
                    Item::Fn(f) => f.sig.ident == "after_each",
                    _ => false,
                });
            let after_each_fn_block = if after_each_fn.len() != 1 {
                panic!("The `after_each` macro attribute requires a single function named `after_each` in the body of the module it is called on.")
            } else {
                match after_each_fn.into_iter().next().unwrap() {
                    Item::Fn(f) => f.block,
                    _ => unreachable!(),
                }
            };

            let e: Vec<Item> = everything_else
                .into_iter()
                .map(|t| match t {
                    Item::Fn(mut f) => {
                        if f.attrs.iter().any(|attr| {
                            attr.path
                                .segments
                                .iter()
                                // only apply after_each biz to functions with test attributes, this
                                // includes variants like `tokio::test`, `test_case`
                                .any(|segment| segment.ident.to_string().contains("test"))
                        }) {
                            f.block.stmts.append(&mut after_each_fn_block.stmts.clone());
                            Item::Fn(f)
                        } else {
                            Item::Fn(f)
                        }
                    }
                    e => e,
                })
                .collect();
            m.content = Some((brace, e));
            Item::Mod(m)
        }

        _ => {
            panic!("The `after_each` macro attribute is only valid when called on a module.")
        }
    };
    TokenStream::from(quote! {#input})
}
