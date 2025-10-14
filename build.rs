fn main() {
    #[cfg(target_os = "none")]
    bare_test_macros::build_test_setup!();
}
