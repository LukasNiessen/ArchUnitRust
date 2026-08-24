struct TestOnlyType {
    enabled: bool,
}

impl TestOnlyType {
    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[test]
fn fixture_is_valid() {
    assert!(TestOnlyType { enabled: true }.is_enabled());
}
