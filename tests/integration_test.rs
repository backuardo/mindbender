use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_encode_decode_without_key() -> Result<(), Box<dyn std::error::Error>> {
    // Set up temporary directories and files
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    // Write sample data
    fs::write(&data_path, "Hello, world!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    // Run the encode command with positional arguments
    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Run the decode command with positional arguments
    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify that decoded text matches original input
    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, "Hello, world!");

    Ok(())
}

#[test]
fn test_encode_decode_with_key() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    let secret_key = "my_secret_key";

    // Write sample data
    fs::write(&data_path, "Secret message!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    // Run the encode command with a key
    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
            "--key",
            secret_key,
        ])
        .assert()
        .success();

    // Run the decode command with the key
    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
            "--key",
            secret_key,
        ])
        .assert()
        .success();

    // Verify that decoded text matches original input
    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, "Secret message!");

    Ok(())
}
