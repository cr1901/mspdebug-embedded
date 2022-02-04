mod cfg;
mod driver;
mod error;

pub use cfg::MspDebugCfg;
pub use driver::MspDebugDriver;
pub use error::MspDebugError;

#[cfg(test)]
mod tests {
    use super::MspDebugCfg;

    // Tests assume mspdebug is on the path.

    #[test]
    fn test_spawn() {
        let mspdebug = MspDebugCfg::new().run();

        assert!(mspdebug.is_ok(), "mspdebug did not spawn: {:?}", unsafe {
            mspdebug.unwrap_err_unchecked()
        });
    }

    #[test]
    fn test_ready() {
        let mut mspdebug = MspDebugCfg::new().run().unwrap();

        let cmd = mspdebug.wait_for_ready();
        assert!(
            cmd.is_ok(),
            "mspdebug did not receive ready: {:?}",
            unsafe { cmd.unwrap_err_unchecked() }
        );
    }
}
