use std::fs;
use tempfile::tempdir;
use nova_launcher_lib::shared::utils::FileVerifier;

#[test]
fn test_file_verifier_sha1() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("test_file.txt");
    
    // Content "Hello Nova Launcher!" has a deterministic SHA-1
    let content = b"Hello Nova Launcher!";
    fs::write(&file_path, content).unwrap();

    let expected_sha1 = "889f60d41cac34d440d3c938a740eb6de5711d31";
    let calculated = FileVerifier::compute_sha1(&file_path).expect("Failed to compute sha1");
    assert_eq!(calculated.to_lowercase(), expected_sha1.to_lowercase());

    // Verify true for matching SHA1
    assert!(FileVerifier::verify_sha1(&file_path, expected_sha1));
    assert!(FileVerifier::verify_sha1(&file_path, &expected_sha1.to_uppercase()));

    // Verify false for incorrect SHA1
    assert!(!FileVerifier::verify_sha1(&file_path, "0000000000000000000000000000000000000000"));

    // Verify false for non-existent file
    assert!(!FileVerifier::verify_sha1(&temp.path().join("non_existent.bin"), expected_sha1));
}