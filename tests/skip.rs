#[cfg(test)]
mod skip {
    use test_env_helpers::*;
    use tokio;

    #[skip]
    mod inner_skip {
        #[test]
        fn test_macro() {
            panic!("mod should be skipped")
        }
    }

    #[skip]
    #[test_case(2)]
    fn async_test_macro_2(_: u8) {
        panic!("test should be skipped")
    }
    #[tokio::test]
    async fn async_test_macro() {}
}
