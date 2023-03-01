mod cfg;
mod driver;
mod error;

pub use cfg::MspDebugCfg;
pub use driver::MspDebugDriver;
pub use error::MspDebugError;

#[cfg(test)]
mod tests {
    use super::cfg;
    use super::MspDebugCfg;
    use serial_test::serial;

    // Tests assume mspdebug is on the path.
    #[test]
    #[serial]
    fn test_spawn() {
        let mspdebug = MspDebugCfg::new().run();

        assert!(mspdebug.is_ok(), "mspdebug did not spawn: {:?}", unsafe {
            mspdebug.unwrap_err_unchecked()
        });
    }

    #[test]
    #[serial]
    fn test_ready() {
        let mut mspdebug = MspDebugCfg::new().run().unwrap();

        let cmd = mspdebug.wait_for_ready();
        assert!(
            cmd.is_ok(),
            "mspdebug did not receive ready: {:?}",
            cmd.unwrap_err()
        );
    }

    // Requires a dev board w/ rf2500- MSP-EXP430G2 is an example.
    mod rf2500 {
        use super::*;

        #[test]
        #[serial]
        fn test_open() {
            let mut mspdebug = MspDebugCfg::new()
                .driver(cfg::Driver::Rf2500)
                .run()
                .unwrap();

            let cmd = mspdebug.wait_for_ready();
            assert!(
                cmd.is_ok(),
                "mspdebug did not receive ready: {:?}",
                cmd.unwrap_err()
            );
        }

        #[test]
        #[serial]
        fn test_prog() {
            let mut mspdebug = MspDebugCfg::new()
                .driver(cfg::Driver::Rf2500)
                .run()
                .unwrap();

            let cmd = mspdebug.program(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/blinky-g2553.elf"
            ));
            assert!(
                cmd.is_ok(),
                "mspdebug could not program ELF file: {:?}",
                cmd.unwrap_err()
            );

            // Program it twice so that we confirm synchronization is working.
            let cmd = mspdebug.program(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/blinky-g2553.elf"
            ));
            assert!(
                cmd.is_ok(),
                "mspdebug could not program ELF file: {:?}",
                cmd.unwrap_err()
            );
        }
    }
}
