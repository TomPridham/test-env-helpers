[![Build Status](https://github.com/tompridham/test-env-helpers/actions/workflows/deploy.yml/badge.svg)](https://github.com/tompridham/test-env-helpers/actions/workflows/deploy.yml)
[![Crates.io](https://img.shields.io/crates/v/test-env-helpers.svg)](https://crates.io/crates/test-env-helpers)
[![Documentation](https://docs.rs/test-env-helpers/badge.svg)](https://docs.rs/test-env-helpers)

## Description
Jest style setup and teardown test helpers.

### Currently implemented:
* `#[after_all]`: Only valid on a `mod`. Requires a single function named `after_all`. Counts the number of functions with a `test` attribute applied and runs the body of the `after_all` function after all the tests have run.
* `#[after_each]`: Only valid on a `mod`. Requires a single function named `after_each`. Copies the body contents of the `after_each` function into the end of the function body of any functions in the same `mod` that have a `test` attribute applied.
* `#[before_all]`: Only valid on a `mod`. Requires a single function named `before_all`. Runs the contents of `before_all` exactly once before any tests have run.
* `#[before_each]`: Only valid on a `mod`. Requires a single function named `before_each`. Copies the body contents of the `before_each` function into the beginning of the function body of any functions in the same `mod` that have `test` attribute applied.
  * N.B. A function with a `test` attribute applied is any function with an attribute with the word `test` in it. So, `#[test]`, `#[tokio::test]`, and `#[test_case(blah)]` will all count for the before/after hooks.

* `#[skip]`: Valid on a `mod` or an individual test. Will skip the mod or test it is applied on.

### To do:
* `#[only]`: Not sure how to implement this one, tbh.
