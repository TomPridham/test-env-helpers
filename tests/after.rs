use test_env_helpers::*;

#[after_all]
#[cfg(test)]
mod after_all {
    use lazy_static::lazy_static;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use test_case::test_case;
    use tokio;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    static T: AtomicUsize = AtomicUsize::new(0);
    fn after_all() {
        assert_eq!(T.load(Ordering::SeqCst), 9);
    }

    #[test]
    fn test_macro() {}
    #[test_case(1)]
    #[test_case(2)]
    fn async_test_macro_2(_: u8) {
        T.fetch_add(3, Ordering::SeqCst);
    }
    #[tokio::test]
    async fn async_test_macro() {
        T.fetch_add(3, Ordering::SeqCst);
    }
}

#[after_each]
#[cfg(test)]
mod after_each {
    use lazy_static::lazy_static;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use test_case::test_case;
    use tokio;

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    static T: AtomicUsize = AtomicUsize::new(0);
    fn after_each() {
        T.store(0, Ordering::Relaxed);
    }

    #[test]
    fn test_macro() {
        let _m = MTX.lock();
        assert_eq!(T.load(Ordering::SeqCst), 0);
    }
    #[test_case(2)]
    fn async_test_macro_2(_: u8) {
        let _m = MTX.lock();
        T.fetch_add(3, Ordering::SeqCst);
        assert_eq!(T.load(Ordering::SeqCst), 3);
    }
    #[tokio::test]
    async fn async_test_macro() {
        let _m = MTX.lock();
        T.fetch_add(3, Ordering::SeqCst);
        assert_eq!(T.load(Ordering::SeqCst), 3);
    }
}
