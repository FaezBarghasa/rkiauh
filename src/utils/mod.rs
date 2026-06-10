pub mod config;
pub mod sys;

#[cfg(test)]
mod tests {
    use super::sys::SysConfigurator;
    use std::fs;

    #[test]
    fn test_udev_rule_generation() {
        let configurer = SysConfigurator::new(false);
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("99-test-pi-ts35.rules");

        let content = r#"SUBSYSTEM=="input", ATTRS{name}=="PI-TS35", ENV{LIBINPUT_CALIBRATION_MATRIX}="1 0 0 0 1 0""#;
        let res = configurer.write_udev_rule(&path, content);
        assert!(res.is_ok());
        assert!(path.exists());
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn test_armbian_env_modification() {
        let configurer = SysConfigurator::new(false);
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("armbianEnv.txt");

        fs::write(&path, "verbosity=1\noverlays=spi-spidev\n").unwrap();

        let res = configurer.write_dt_overlays(&path, &["pi-ts35"]);
        assert!(res.is_ok());

        let result_content = fs::read_to_string(&path).unwrap();
        assert!(result_content.contains("overlays=spi-spidev pi-ts35"));

        fs::remove_file(&path).unwrap();
    }
}