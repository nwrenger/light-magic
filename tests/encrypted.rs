use std::{fs, path::Path};

use light_magic::{
    encrypted::EncryptedDataStore,
    serde::{Deserialize, Serialize},
};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    items: Vec<String>,
}

impl EncryptedDataStore for TestData {}

/// Helper struct that deletes the file when dropped
struct TempDbPath {
    path: String,
}

impl TempDbPath {
    fn new(test_name: &str) -> Self {
        let path = format!("./tests/{}.db", test_name);

        // Ensure tests dir exists
        let _ = fs::create_dir_all("./tests");

        // Remove if it somehow already exists
        let _ = fs::remove_file(&path);

        TempDbPath { path }
    }

    fn as_str(&self) -> &str {
        &self.path
    }
}

impl Drop for TempDbPath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

const PASSWORD: &'static str = "securepassword";

#[test]
fn creation() {
    let db_path = TempDbPath::new("creation");

    // Create and populate the database
    {
        let db = TestData::open(db_path.as_str(), PASSWORD).expect("Failed to create database");
        let mut data = db.write();
        data.items
            .extend(vec!["Item 1".to_string(), "Item 2".to_string()]);
    }

    // Ensure the database file was created
    assert!(
        Path::new(db_path.as_str()).exists(),
        "Database file was not created"
    );

    // Load and verify the database content
    {
        let db = TestData::open(db_path.as_str(), PASSWORD).expect("Failed to load database");
        let data = db.read();
        assert_eq!(
            *data,
            TestData {
                items: vec!["Item 1".to_string(), "Item 2".to_string()]
            },
            "Data was not saved and loaded correctly"
        );
    }
}

#[test]
fn multiple_saves() {
    let db_path = TempDbPath::new("multiple_saves");

    // Test multiple saves
    {
        let db = TestData::open(db_path.as_str(), PASSWORD)
            .expect("Failed to open database for multiple saves");
        for i in 1..=3 {
            let mut data = db.write();
            data.items.push(format!("Item {}", i));
        }
    }

    // Verify multiple saves
    {
        let db = TestData::open(db_path.as_str(), PASSWORD)
            .expect("Failed to load database after multiple saves");
        let data = db.read();
        let expected_items: Vec<String> = (1..=3).map(|i| format!("Item {}", i)).collect();
        assert_eq!(
            *data,
            TestData {
                items: expected_items
            },
            "Not all items were saved correctly"
        );
    }
}

#[test]
fn password_change() {
    let db_path = TempDbPath::new("password_change");

    // Define new password
    let new_password = "newsecurepassword";

    // Change the password from initial_password to new_password
    {
        // Open the database
        let db = TestData::open(db_path.as_str(), PASSWORD)
            .expect("Failed to open database for password change");

        // Perform password change
        db.change_password(new_password)
            .expect("Failed to change password");
    }

    // Verify that the database can be opened with the new password
    {
        let db = TestData::open(db_path.as_str(), new_password)
            .expect("Failed to load database with new password");
        let data_new = db.read();
        let expected_items: Vec<String> = vec![];
        assert_eq!(
            *data_new,
            TestData {
                items: expected_items
            },
            "Data was not preserved after password change"
        );
    }

    // Attempt to load the database with the old password (should fail)
    {
        let result = TestData::open(db_path.as_str(), PASSWORD);
        assert!(
            result.is_err(),
            "Loading with old password should have failed after password change"
        );
        if let Err(e) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
            assert!(
                e.to_string().contains("Decryption failed")
                    || e.to_string()
                        .contains("HMAC verification failed: Data is corrupted or tampered"),
                "Error message does not indicate decryption failure"
            );
        }
    }
}

#[test]
fn wrong_password() {
    let db_path = TempDbPath::new("wrong_password");
    let wrong_password = "wrongpassword";

    // Create and populate the database
    {
        let db = TestData::open(db_path.as_str(), PASSWORD).expect("Failed to create database");
        let mut data = db.write();
        data.items.push("Valid Item".to_string());
    }

    // Attempt to load with the wrong password
    {
        let result = TestData::open(db_path.as_str(), wrong_password);
        assert!(
            result.is_err(),
            "Loading with wrong password should have failed"
        );
        if let Err(e) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
            assert!(
                e.to_string().contains("Decryption failed")
                    || e.to_string()
                        .contains("HMAC verification failed: Data is corrupted or tampered"),
                "Error message does not indicate decryption failure"
            );
        }
    }

    // Ensure data integrity by loading with the new password again
    {
        let db = TestData::open(db_path.as_str(), PASSWORD)
            .expect("Failed to load database with new password again");
        let data = db.read();
        let expected_items = vec![String::from("Valid Item")];
        assert_eq!(
            *data,
            TestData {
                items: expected_items
            },
            "Data was not preserved on subsequent loads after password change"
        );
    }
}

#[test]
fn file_corruption() {
    let db_path = TempDbPath::new("file_corruption");

    // create db
    {
        let _ = TestData::open(db_path.as_str(), PASSWORD).unwrap();
    }

    // Corrupt the database file
    fs::write(db_path.as_str(), "corrupted data").expect("Failed to write corrupted data");

    // Attempt to load the corrupted file
    let corrupted_result = TestData::open(db_path.as_str(), PASSWORD);
    assert!(
        corrupted_result.is_err(),
        "Loading a corrupted file should have failed"
    );
    if let Err(e) = corrupted_result {
        dbg!(e.to_string());
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            e.to_string()
                .contains("Failed to deserialize encrypted data")
                || e.to_string().contains("Decryption failed"),
            "Error message does not indicate corruption"
        );
    }
}
