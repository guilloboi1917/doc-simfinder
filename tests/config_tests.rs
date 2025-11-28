use doc_simfinder::config::Config;
use std::path::PathBuf;

#[test]
fn test_config_validate_empty_query() {
    let mut cfg = Config::default();
    cfg.search_path = PathBuf::from("testdata");
    cfg.query = "".to_string();
    assert!(cfg.validate().is_err());
}

#[test]
fn test_config_validate_ok() {
    let mut cfg = Config::default();
    cfg.search_path = PathBuf::from("testdata");
    cfg.query = "x".to_string();
    assert!(cfg.validate().is_ok());
}
