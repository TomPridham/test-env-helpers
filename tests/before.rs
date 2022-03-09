use test_env_helpers::*;

#[before_all]
#[cfg(test)]
mod before_all {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use test_case::test_case;
    use tokio;

    static T: AtomicUsize = AtomicUsize::new(0);
    fn before_all() {
        assert_eq!(T.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_macro() {
        T.fetch_add(3, Ordering::SeqCst);
    }
    #[test_case(2)]
    fn async_test_macro_2(_: u8) {
        T.fetch_add(3, Ordering::SeqCst);
    }
    #[tokio::test]
    async fn async_test_macro() {
        T.fetch_add(3, Ordering::SeqCst);
    }
}

#[before_each]
#[cfg(test)]
mod before_each {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use test_case::test_case;
    use tokio;

    static T: AtomicUsize = AtomicUsize::new(0);
    fn before_each() {
        T.store(1, Ordering::Relaxed);
    }

    #[test]
    fn test_macro() {
        assert_eq!(T.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn async_test_macro() {
        T.fetch_add(3, Ordering::SeqCst);
        assert_eq!(T.load(Ordering::SeqCst), 4);
    }
    #[test_case(2)]
    fn async_test_macro_2(_: u8) {
        T.fetch_add(3, Ordering::SeqCst);
        assert_eq!(T.load(Ordering::SeqCst), 4);
    }
}
