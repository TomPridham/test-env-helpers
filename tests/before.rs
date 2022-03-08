use test_env_helpers::*;

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
