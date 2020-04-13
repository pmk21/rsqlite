use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn insert_single_row() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rsqlite").unwrap();
    cmd.write_stdin("insert 1 alice foo@example.com\n.exit\n")
        .assert()
        .success()
        .stdout(predicate::eq("db > Executed.\ndb > "));

    Ok(())
}

#[test]
fn insert_more_than_max_rows() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rsqlite").unwrap();
    let mut cmd_str = String::new();

    for i in 1..1402 {
        cmd_str.push_str(&format!("insert {} user{} person{}@example.com\n", i, i, i));
    }

    cmd_str.push_str(".exit\n");

    let assert = cmd.write_stdin(cmd_str).assert();
    let output_str = String::from_utf8(assert.success().get_output().stdout.clone()).unwrap();
    let op: Vec<&str> = output_str.split('\n').collect();

    assert_eq!(op[op.len() - 2], "db > Error: Table full.");

    Ok(())
}

#[test]
fn insert_max_length_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rsqlite").unwrap();
    let long_username = "a".repeat(32);
    let long_email = "a".repeat(255);
    let op_str = &format!("db > (1, {}, {})", long_username, long_email);
    let expected_op: Vec<&str> = vec!["db > Executed.", op_str, "Executed.", "db > "];

    let assert = cmd
        .write_stdin(format!(
            "insert 1 {} {}\nselect\n.exit\n",
            long_username, long_email
        ))
        .assert();

    let output_str = String::from_utf8(assert.success().get_output().stdout.clone()).unwrap();
    let op: Vec<&str> = output_str.split('\n').collect();

    assert_eq!(op, expected_op);

    Ok(())
}

#[test]
fn insert_large_fields() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rsqlite").unwrap();
    let long_username = "a".repeat(33);
    let long_email = "a".repeat(256);
    let expected_op: Vec<&str> = vec!["db > String is too long.", "db > Executed.", "db > "];

    let assert = cmd
        .write_stdin(format!(
            "insert 1 {} {}\nselect\n.exit\n",
            long_username, long_email
        ))
        .assert();

    let output_str = String::from_utf8(assert.success().get_output().stdout.clone()).unwrap();
    let op: Vec<&str> = output_str.split('\n').collect();

    assert_eq!(op, expected_op);

    Ok(())
}

#[test]
fn insert_negative_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rsqlite").unwrap();
    let expected_op: Vec<&str> = vec!["db > ID must be positive.", "db > Executed.", "db > "];

    let assert = cmd
        .write_stdin("insert -1 test test@example.com\nselect\n.exit\n")
        .assert();

    let output_str = String::from_utf8(assert.success().get_output().stdout.clone()).unwrap();
    let op: Vec<&str> = output_str.split('\n').collect();

    assert_eq!(op, expected_op);

    Ok(())
}
