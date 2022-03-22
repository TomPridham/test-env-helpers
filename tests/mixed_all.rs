#![allow(unused_imports)]
use test_env_helpers::*;

#[before_all]
#[after_all]
#[cfg(test)]
mod before_after {
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod once_import {
    use std::sync::Once;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod sync_glob_import {
    use std::sync::*;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod atomic_glob_import {
    use std::sync::atomic::*;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod atomic_usize_import {
    use std::sync::atomic::AtomicUsize;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod atomic_ordering_import {
    use std::sync::atomic::Ordering;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}

#[before_all]
#[after_all]
#[cfg(test)]
mod other_atomic_import {
    use std::sync::atomic::AtomicBool;
    fn before_all() {}
    fn after_all() {}

    #[test]
    fn test_macro() {}
}
