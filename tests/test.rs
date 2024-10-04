use light_magic::{
    atomic::DataStore,
    encrypted::EncryptedDataStore,
    join,
    serde::{Deserialize, Serialize},
    table::{PrimaryKey, Table},
};

use std::fs;
use std::path::Path;

#[derive(Default, Debug, Serialize, Deserialize)]
struct Database {
    users: Table<User>,
    permissions: Table<Permission>,
    criminals: Table<Criminal>,
    settings: Settings,
}

impl DataStore for Database {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: usize,
    name: String,
    kind: String,
}

impl PrimaryKey for User {
    type PrimaryKeyType = usize;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.id
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Permission {
    user_name: String,
    level: Level,
}

impl PrimaryKey for Permission {
    type PrimaryKeyType = String;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.user_name
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
enum Level {
    #[default]
    Admin,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Criminal {
    user_name: String,
    entry: String,
}

impl PrimaryKey for Criminal {
    type PrimaryKeyType = String;

    fn primary_key(&self) -> &Self::PrimaryKeyType {
        &self.user_name
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Settings {
    time: usize,
    password: String,
}

#[test]
fn normal_ops_in_persistence() {
    let db = Database::open("./tests/test.json");

    // normal adding
    db.write().users.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    assert!(db.read().users.get(&0).is_some());
    assert!(
        db.read()
            .users
            .search(|user| { user.name.contains("Nils") })
            .len()
            == 1
    );

    db.write().permissions.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    assert!(db.read().permissions.get(&String::from("Nils")).is_some());
    assert!(
        db.read()
            .permissions
            .search(|permission| { permission.level == Level::Admin })
            .len()
            == 1
    );

    db.write().criminals.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    assert!(db.read().criminals.get(&String::from("Nils")).is_some());
    assert!(
        db.read()
            .criminals
            .search(|criminal| { criminal.entry.contains("No records") })
            .len()
            == 1
    );

    let settings = Settings {
        time: 1718744090,
        password: String::from("password"),
    };

    db.write().settings = settings.clone();

    assert_eq!(db.read().settings, settings);

    // editing
    db.write().criminals.edit(
        &String::from("Nils"),
        Criminal {
            user_name: String::from("Nils W."),
            entry: String::from("Stole a hot dog!"),
        },
    );

    assert!(db.read().criminals.get(&String::from("Nils")).is_none());
    assert!(db.read().criminals.get(&String::from("Nils W.")).is_some());
    assert!(
        db.read()
            .criminals
            .search(|criminal| { criminal.entry.contains("hot dog") })
            .len()
            == 1
    );

    // deleting
    db.write().criminals.delete(&String::from("Nils W."));

    assert!(db.read().criminals.get(&String::from("Nils W.")).is_none());
}

#[test]
fn normal_ops_in_memory() {
    let db = Database::open_in_memory();

    // normal adding
    db.write().users.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });
    assert!(db.read().users.get(&0).is_some());
    assert!(
        db.read()
            .users
            .search(|user| { user.name.contains("Nils") })
            .len()
            == 1
    );

    db.write().permissions.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });
    assert!(db.read().permissions.get(&String::from("Nils")).is_some());
    assert!(
        db.read()
            .permissions
            .search(|permission| { permission.level == Level::Admin })
            .len()
            == 1
    );

    db.write().criminals.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });
    assert!(db.read().criminals.get(&String::from("Nils")).is_some());
    assert!(
        db.read()
            .criminals
            .search(|criminal| { criminal.entry.contains("No records") })
            .len()
            == 1
    );

    let settings = Settings {
        time: 1718744090,
        password: String::from("password"),
    };

    db.write().settings = settings.clone();

    assert_eq!(db.read().settings, settings);

    // editing
    db.write().criminals.edit(
        &String::from("Nils"),
        Criminal {
            user_name: String::from("Nils W."),
            entry: String::from("Stole a hot dog!"),
        },
    );

    assert!(db.read().criminals.get(&String::from("Nils")).is_none());
    assert!(db.read().criminals.get(&String::from("Nils W.")).is_some());
    assert!(
        db.read()
            .criminals
            .search(|criminal| { criminal.entry.contains("hot dog") })
            .len()
            == 1
    );

    // deleting
    db.write().criminals.delete(&String::from("Nils W."));

    assert!(db.read().criminals.get(&String::from("Nils W.")).is_none());
}

#[test]
fn custom_derives() {
    let db = Database::open_in_memory();

    let user1 = User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    };
    let user2 = User {
        id: 1,
        name: String::from("Alice"),
        kind: String::from("Old"),
    };

    let inserted_user1 = db.write().users.add(user1.clone()).unwrap_or(user1);
    let inserted_user2 = db.write().users.add(user2.clone()).unwrap_or(user2);

    assert_ne!(inserted_user1, inserted_user2);

    assert_eq!(inserted_user1.name, "Nils");
    assert_eq!(inserted_user2.name, "Alice");
}

#[test]
fn joins() {
    let db = Database::open_in_memory();

    // remove any if there
    db.write().permissions.delete(&"Nils".to_string());
    db.write().criminals.delete(&"Nils".to_string());

    // add smth
    db.write().users.add(User {
        id: 0,
        name: String::from("Nils"),
        kind: String::from("Young"),
    });

    let joined =
        join!(db.read(), "Nils", users => name, permissions => user_name, criminals => user_name);
    assert!(joined.is_empty());

    // add more
    db.write().permissions.add(Permission {
        user_name: String::from("Nils"),
        level: Level::Admin,
    });

    db.write().criminals.add(Criminal {
        user_name: String::from("Nils"),
        entry: String::from("No records until this day! Keep ur eyes pealed!"),
    });

    let joined =
        join!(db.read(), "Nils", users => name, permissions => user_name, criminals => user_name);
    assert!(!joined.is_empty());

    // add even more
    for i in 0..4 {
        db.write().users.add(User {
            id: i,
            name: String::from("Smth".to_string() + &i.to_string()),
            kind: String::from("Young"),
        });

        db.write().permissions.add(Permission {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            level: Level::Admin,
        });

        db.write().criminals.add(Criminal {
            user_name: String::from("Smth".to_string() + &i.to_string()),
            entry: String::from("No records until this day! Keep ur eyes pealed!"),
        });
    }

    let joined =
        join!(db.read(), "Smth2", users => name, permissions => user_name, criminals => user_name);

    assert!(joined.len() == 1);
    assert!(joined[0].0.name == "Smth2");
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    items: Vec<String>,
}

impl EncryptedDataStore for TestData {}

/// Helper function to remove the encrypted database file before each test
fn remove_encrypted_db(path: &str) {
    if Path::new(path).exists() {
        fs::remove_file(path).expect("Failed to remove existing encrypted database");
    }
}

#[test]
fn persistent_encrypted_db() {
    // ================== Database Creation && Saving/Loading ==================

    let initial_password = "securepassword";
    let db_path = "./tests/test_encrypted.json";
    remove_encrypted_db(db_path);

    // Create and populate the database
    {
        let db = TestData::open(db_path, initial_password).expect("Failed to create database");
        let mut data = db.write();
        data.items
            .extend(vec!["Item 1".to_string(), "Item 2".to_string()]);
    }

    // Ensure the database file was created
    assert!(Path::new(db_path).exists(), "Database file was not created");

    // Load and verify the database content
    let db_loaded = TestData::open(db_path, initial_password).expect("Failed to load database");
    let data = db_loaded.read();
    assert_eq!(
        *data,
        TestData {
            items: vec!["Item 1".to_string(), "Item 2".to_string()]
        },
        "Data was not saved and loaded correctly"
    );

    // Test multiple saves
    {
        let db = TestData::open(db_path, initial_password)
            .expect("Failed to open database for multiple saves");
        for i in 3..=5 {
            let mut data = db.write();
            data.items.push(format!("Item {}", i));
        }
    }

    // Verify multiple saves
    let db_loaded = TestData::open(db_path, initial_password)
        .expect("Failed to load database after multiple saves");
    let data = db_loaded.read();
    let expected_items: Vec<String> = (1..=5).map(|i| format!("Item {}", i)).collect();
    assert_eq!(
        *data,
        TestData {
            items: expected_items
        },
        "Not all items were saved correctly"
    );

    // ================== Password Change ==================

    // Define new password
    let new_password = "newsecurepassword";

    // Change the password from initial_password to new_password
    {
        // Open the database
        let db = TestData::open(db_path, initial_password)
            .expect("Failed to open database for password change");

        // Perform password change
        db.change_password(new_password)
            .expect("Failed to change password");
    }

    // Verify that the database can be opened with the new password
    let db_loaded_new =
        TestData::open(db_path, new_password).expect("Failed to load database with new password");
    let data_new = db_loaded_new.read();
    let expected_items_new: Vec<String> = (1..=5).map(|i| format!("Item {}", i)).collect();
    assert_eq!(
        *data_new,
        TestData {
            items: expected_items_new.clone()
        },
        "Data was not preserved after password change"
    );

    // Attempt to load the database with the old password (should fail)
    let db_loaded_old = TestData::open(db_path, initial_password);
    assert!(
        db_loaded_old.is_err(),
        "Loading with old password should have failed after password change"
    );
    if let Err(e) = db_loaded_old {
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            e.to_string().contains("Decryption failed")
                || e.to_string()
                    .contains("Failed to deserialize decrypted data"),
            "Error message does not indicate decryption failure"
        );
    }

    // Ensure data integrity by loading with the new password again
    let db_loaded_new_again = TestData::open(db_path, new_password)
        .expect("Failed to load database with new password again");
    let data_new_again = db_loaded_new_again.read();
    assert_eq!(
        *data_new_again,
        TestData {
            items: expected_items_new
        },
        "Data was not preserved on subsequent loads after password change"
    );

    // ================== Wrong Password && Data Corruption ==================

    let correct_password = "securepassword";
    let wrong_password = "wrongpassword";
    let db_path = "./tests/test_encrypted.json";
    remove_encrypted_db(db_path);

    // Create and populate the database
    {
        let db = TestData::open(db_path, correct_password).expect("Failed to create database");
        let mut data = db.write();
        data.items.push("Valid Item".to_string());
    }

    // Attempt to load with the wrong password
    let wrong_password_result = TestData::open(db_path, wrong_password);
    assert!(
        wrong_password_result.is_err(),
        "Loading with wrong password should have failed"
    );
    if let Err(e) = wrong_password_result {
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            e.to_string().contains("Decryption failed")
                || e.to_string()
                    .contains("Failed to deserialize decrypted data"),
            "Error message does not indicate decryption failure"
        );
    }

    // Corrupt the database file
    fs::write(db_path, "corrupted data").expect("Failed to write corrupted data");

    // Attempt to load the corrupted file
    let corrupted_result = TestData::open(db_path, correct_password);
    assert!(
        corrupted_result.is_err(),
        "Loading a corrupted file should have failed"
    );
    if let Err(e) = corrupted_result {
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            e.to_string()
                .contains("Failed to deserialize encrypted data")
                || e.to_string().contains("Decryption failed"),
            "Error message does not indicate corruption"
        );
    }
}
