//
//! Some setup and teardown macro helpers to mimic [Jest's setup and teardown](https://jestjs.io/docs/setup-teardown)
//! functionality. Also includes a `skip` macro that mimics the [skip](https://jestjs.io/docs/api#testskipname-fn)
//! functionality in Jest.
//!
//! There are currently five macros provided: `after_all`,
//! `after_each`, `before_all`, `before_each`, and `skip`. I would like to implement `only` to
//! match [Jest's only](https://jestjs.io/docs/api#testonlyname-fn-timeout) functionality. I'm
//! unsure of a great way to do that currently, however.
//!
//! ## Getting Started
//! Using these macros is fairly simple. The four after/before functions all require a function
//! with the same name as the attribute and are only valid when applied to a mod. They are all used
//! like in the below example. Replace `before_each` with whichever method you want to use. The
//! code in the matching function will be inserted into every fn in the containing mod that has an
//! attribute with the word "test" in it. This is to allow for use with not just normal `#[test]`
//! attributes, but also other flavors like `#[tokio::test]` and `#[test_case(0)]`.
//! ```
//! #[cfg(test)]
//! use test_env_helpers::*;
//!
//! #[before_each]
//! #[cfg(test)]
//! mod my_tests{
//!     fn before_each(){println!("I'm in every test!")}
//!     #[test]
//!     fn test_1(){}
//!     #[test]
//!     fn test_2(){}
//!     #[test]
//!     fn test_3(){}
//! }
//! ```
//!
//! The `skip` macro is valid on either a mod or an individual test and will remove the mod or test
//! it is applied to. You can use it to skip tests that aren't working correctly or that you don't
//! want to run for some reason.
//!
//! ```
//! #[cfg(test)]
//! use test_env_helpers::*;
//!
//! #[cfg(test)]
//! mod my_tests{
//!     #[skip]
//!     #[test]
//!     fn broken_test(){panic!("I'm hella broke")}
//!     #[skip]
//!     mod broken_mod{
//!         #[test]
//!         fn i_will_not_be_run(){panic!("I get skipped too")}
//!     }
//!     #[test]
//!     fn test_2(){}
//!     #[test]
//!     fn test_3(){}
//! }
//! ```

extern crate proc_macro;
mod utils;

use crate::utils::traverse_use_item;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item, Stmt};

/// Will run the code in the matching `after_all` function exactly once when all of the tests have
/// run. This works by counting the number of `#[test]` attributes and decrementing a counter at
/// the beginning of every test. Once the counter reaches 0, it will run the code in `after_all`.
/// It uses [std::sync::Once](https://doc.rust-lang.org/std/sync/struct.Once.html) internally
/// to ensure that the code is run at maximum one time.
///
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[after_all]
/// #[cfg(test)]
/// mod my_tests{
///     fn after_all(){println!("I only get run once at the very end")}
///     #[test]
///     fn test_1(){}
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
#[proc_macro_attribute]
pub fn after_all(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input: Item = match parse_macro_input!(input as Item) {
        Item::Mod(mut m) => {
            let (brace, items) = m.content.unwrap();
            let (after_all_fn, everything_else): (Vec<Item>, Vec<Item>) =
                items.into_iter().partition(|t| match t {
                    Item::Fn(f) => f.sig.ident == "after_all",
                    _ => false,
                });
            let after_all_fn_block = if after_all_fn.len() != 1 {
                panic!("The `after_all` macro attribute requires a single function named `after_all` in the body of the module it is called on.")
            } else {
                match after_all_fn.into_iter().next().unwrap() {
                    Item::Fn(f) => f.block,
                    _ => unreachable!(),
                }
            };
            let after_all_if: Stmt = parse_quote! {
                if REMAINING_TESTS.fetch_sub(1, Ordering::SeqCst) == 1 {
                    AFTER_ALL.call_once(|| {
                        #after_all_fn_block
                    });
                }
            };

            let mut count: usize = 0;
            let mut has_once: bool = false;
            let mut has_atomic_usize: bool = false;
            let mut has_ordering: bool = false;

            let mut e: Vec<Item> = everything_else
                .into_iter()
                .map(|t| match t {
                    Item::Fn(mut f) => {
                        let test_count = f
                            .attrs
                            .iter()
                            .filter(|attr| {
                                attr.path
                                    .segments
                                    .iter()
                                    .any(|segment| segment.ident.to_string().contains("test"))
                            })
                            .count();
                        if test_count > 0 {
                            count += test_count;
                            let mut stmts = vec![];
                            stmts.append(&mut f.block.stmts);
                            stmts.push(after_all_if.clone());
                            f.block.stmts = stmts;
                            Item::Fn(f)
                        } else {
                            Item::Fn(f)
                        }
                    }
                    Item::Use(use_stmt) => {
                        if traverse_use_item(&use_stmt.tree, vec!["std", "sync", "Once"]).is_some()
                        {
                            has_once = true;
                        }
                        if traverse_use_item(
                            &use_stmt.tree,
                            vec!["std", "sync", "atomic", "AtomicUsize"],
                        )
                        .is_some()
                        {
                            has_atomic_usize = true;
                        }
                        if traverse_use_item(
                            &use_stmt.tree,
                            vec!["std", "sync", "atomic", "Ordering"],
                        )
                        .is_some()
                        {
                            has_ordering = true;
                        }
                        Item::Use(use_stmt)
                    }
                    el => el,
                })
                .collect();

            let use_once: Item = parse_quote!(
                use std::sync::Once;
            );
            let use_atomic_usize: Item = parse_quote!(
                use std::sync::atomic::AtomicUsize;
            );
            let use_ordering: Item = parse_quote!(
                use std::sync::atomic::Ordering;
            );
            let static_once: Item = parse_quote!(
                static AFTER_ALL: Once = Once::new();
            );
            let static_count: Item = parse_quote!(
                static REMAINING_TESTS: AtomicUsize = AtomicUsize::new(#count);
            );

            let mut once_content = vec![];

            if !has_once {
                once_content.push(use_once);
            }
            if !has_atomic_usize {
                once_content.push(use_atomic_usize);
            }
            if !has_ordering {
                once_content.push(use_ordering);
            }
            once_content.append(&mut vec![static_once, static_count]);
            once_content.append(&mut e);

            m.content = Some((brace, once_content));
            Item::Mod(m)
        }
        _ => {
            panic!("The `after_all` macro attribute is only valid when called on a module.")
        }
    };
    TokenStream::from(quote! (#input))
}

/// Will run the code in the matching `after_each` function at the end of every `#[test]` function.
/// Useful if you want to cleanup after a test or reset some external state. If the test panics,
/// this code will not be run. If you need something that is infallible, you should use
/// `before_each` instead.
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[after_each]
/// #[cfg(test)]
/// mod my_tests{
///     fn after_each(){println!("I get run at the very end of each function")}
///     #[test]
///     fn test_1(){}
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
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

/// Will run the code in the matching `before_all` function exactly once at the very beginning of a
/// test run. It uses [std::sync::Once](https://doc.rust-lang.org/std/sync/struct.Once.html) internally
/// to ensure that the code is run at maximum one time. Useful for setting up some external state
/// that will be reused in multiple tests.
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[before_all]
/// #[cfg(test)]
/// mod my_tests{
///     fn before_all(){println!("I get run at the very beginning of the test suite")}
///     #[test]
///     fn test_1(){}
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
#[proc_macro_attribute]
pub fn before_all(_metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input: Item = match parse_macro_input!(input as Item) {
        Item::Mod(mut m) => {
            let (brace, items) = m.content.unwrap();
            let (before_all_fn, everything_else): (Vec<Item>, Vec<Item>) =
                items.into_iter().partition(|t| match t {
                    Item::Fn(f) => f.sig.ident == "before_all",
                    _ => false,
                });
            let before_all_fn_block = if before_all_fn.len() != 1 {
                panic!("The `before_all` macro attribute requires a single function named `before_all` in the body of the module it is called on.")
            } else {
                match before_all_fn.into_iter().next().unwrap() {
                    Item::Fn(f) => f.block,
                    _ => unreachable!(),
                }
            };
            let q: Stmt = parse_quote! {
                BEFORE_ALL.call_once(|| {
                    #before_all_fn_block
                });
            };

            let mut has_once: bool = false;
            let mut e: Vec<Item> = everything_else
                .into_iter()
                .map(|t| match t {
                    Item::Fn(mut f) => {
                        if f.attrs.iter().any(|attr| {
                            attr.path
                                .segments
                                .iter()
                                .any(|segment| segment.ident.to_string().contains("test"))
                        }) {
                            let mut stmts = vec![q.clone()];
                            stmts.append(&mut f.block.stmts);
                            f.block.stmts = stmts;
                            Item::Fn(f)
                        } else {
                            Item::Fn(f)
                        }
                    }
                    Item::Use(use_stmt) => {
                        if traverse_use_item(&use_stmt.tree, vec!["std", "sync", "Once"]).is_some()
                        {
                            has_once = true;
                        }
                        Item::Use(use_stmt)
                    }
                    e => e,
                })
                .collect();
            let use_once: Item = parse_quote!(
                use std::sync::Once;
            );
            let static_once: Item = parse_quote!(
                static BEFORE_ALL: Once = Once::new();
            );

            let mut once_content = vec![];
            if !has_once {
                once_content.push(use_once);
            }
            once_content.push(static_once);
            once_content.append(&mut e);

            m.content = Some((brace, once_content));
            Item::Mod(m)
        }

        _ => {
            panic!("The `before_all` macro attribute is only valid when called on a module.")
        }
    };
    TokenStream::from(quote! (#input))
}

/// Will run the code in the matching `before_each` function at the beginning of every test. Useful
/// to reset state to ensure that a test has a clean slate.
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[before_each]
/// #[cfg(test)]
/// mod my_tests{
///     fn before_each(){println!("I get run at the very beginning of every test")}
///     #[test]
///     fn test_1(){}
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
///
/// Can be used to reduce the amount of boilerplate setup code that needs to be copied into each test.
/// For example, if you need to ensure that tests in a single test suite are not run in parallel, this can
/// easily be done with a [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
/// However, remembering to copy and paste the code to acquire a lock on the `Mutex` in every test
/// is tedious and error prone.
/// ```
/// #[cfg(test)]
/// mod without_before_each{
///     lazy_static! {
///         static ref MTX: Mutex<()> = Mutex::new(());
///     }
///     #[test]
///     fn test_1(){let _m = MTX.lock();}
///     #[test]
///     fn test_2(){let _m = MTX.lock();}
///     #[test]
///     fn test_3(){let _m = MTX.lock();}
/// }
/// ```
/// Using `before_each` removes the need to copy and paste so much and makes making changes easier
/// because they only need to be made in a single location instead of once for every test.
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[before_each]
/// #[cfg(test)]
/// mod with_before_each{
///     lazy_static! {
///         static ref MTX: Mutex<()> = Mutex::new(());
///     }
///     fn before_each(){let _m = MTX.lock();}
///     #[test]
///     fn test_1(){}
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
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

/// Will skip running the code it is applied on. You can use it to skip tests that aren't working
/// correctly or that you don't want to run for some reason. There are no checks to make sure it's
/// applied to a `#[test]` or mod. It will remove whatever it is applied to from the final AST.
///
/// ```
/// #[cfg(test)]
/// use test_env_helpers::*;
///
/// #[cfg(test)]
/// mod my_tests{
///     #[skip]
///     #[test]
///     fn broken_test(){panic!("I'm hella broke")}
///     #[skip]
///     mod broken_mod{
///         #[test]
///         fn i_will_not_be_run(){panic!("I get skipped too")}
///     }
///     #[test]
///     fn test_2(){}
///     #[test]
///     fn test_3(){}
/// }
/// ```
#[proc_macro_attribute]
pub fn skip(_metadata: TokenStream, _input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {})
}
