use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_encode_decode_without_key() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    fs::write(&data_path, "Hello, world!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

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

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
        ])
        .assert()
        .success();

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

    fs::write(&data_path, "Secret message!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

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

    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, "Secret message!");

    Ok(())
}

#[test]
fn test_encode_decode_with_lossy_image() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.jpg");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    fs::write(&data_path, "Message in lossy image!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.jpeg"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Warning: Carrier image is lossy. Converting to lossless format...",
        ));

    assert!(encoded_image_path.exists());
    assert_eq!(
        encoded_image_path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase(),
        "png"
    );

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, "Message in lossy image!");

    Ok(())
}

#[test]
fn test_decode_with_incorrect_key() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    let correct_key = "correct_secret_key";
    let incorrect_key = "incorrect_secret_key";

    fs::write(&data_path, "Secret message!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
            "--key",
            correct_key,
        ])
        .assert()
        .success();

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
            "--key",
            incorrect_key,
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Decryption error"));

    Ok(())
}

#[test]
fn test_encode_decode_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    let special_message = "特殊字符测试 🚀✨";

    fs::write(&data_path, special_message)?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

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

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, special_message);

    Ok(())
}

#[test]
fn test_encode_overwrites_existing_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");

    fs::write(&data_path, "Hello, world!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;
    fs::write(&encoded_image_path, "Existing file content")?;

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

    let metadata = fs::metadata(&encoded_image_path)?;
    assert!(metadata.len() > 0);
    let new_content = fs::read(&encoded_image_path)?;
    assert_ne!(new_content, b"Existing file content");

    Ok(())
}

#[test]
fn test_encode_with_non_image_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("not_an_image.txt");
    let encoded_image_path = temp_dir.path().join("encoded.png");

    fs::write(&data_path, "Hello, world!")?;
    fs::write(&carrier_path, "This is not an image.")?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid path error"));

    Ok(())
}

#[test]
fn test_encode_with_insufficient_capacity() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("large_data.txt");
    let carrier_path = temp_dir.path().join("carrier_small.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");

    fs::write(
        &data_path,
        "This message is too long for the carrier image.",
    )?;
    fs::write(&carrier_path, include_bytes!("example/carrier_small.png"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Encoding error: Image too small to encode data",
        ));

    Ok(())
}

#[test]
fn test_encode_decode_with_compression() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    fs::write(&data_path, "Message with compression!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
            "--compress",
        ])
        .assert()
        .success();

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
            "--decompress",
        ])
        .assert()
        .success();

    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, "Message with compression!");

    Ok(())
}

#[test]
fn test_encode_with_compression_decode_without_decompression(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    fs::write(&data_path, "Compressed message!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
            "--compress",
        ])
        .assert()
        .success();

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Data is compressed but decompression was not requested",
        ));

    Ok(())
}

#[test]
fn test_decode_without_compression_with_decompression() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    fs::write(&data_path, "Non-compressed message!")?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

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

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
            "--decompress",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Decompression expected, but message is not compressed",
        ));

    Ok(())
}

#[test]
fn test_compression_decompression_large_data() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_path = temp_dir.path().join("data.txt");
    let carrier_path = temp_dir.path().join("carrier.png");
    let encoded_image_path = temp_dir.path().join("encoded.png");
    let decoded_text_path = temp_dir.path().join("decoded.txt");

    let large_message = "Large message!".repeat(1000);
    fs::write(&data_path, &large_message)?;
    fs::write(&carrier_path, include_bytes!("example/carrier.png"))?;

    Command::cargo_bin("mindbender")?
        .args(&[
            "encode",
            data_path.to_str().unwrap(),
            carrier_path.to_str().unwrap(),
            "--output-path",
            encoded_image_path.to_str().unwrap(),
            "--compress",
        ])
        .assert()
        .success();

    Command::cargo_bin("mindbender")?
        .args(&[
            "decode",
            encoded_image_path.to_str().unwrap(),
            "--output-path",
            decoded_text_path.to_str().unwrap(),
            "--decompress",
        ])
        .assert()
        .success();

    let decoded_text = fs::read_to_string(decoded_text_path)?;
    assert_eq!(decoded_text, large_message);

    Ok(())
}
