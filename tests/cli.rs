use assert_cmd::Command;
use predicates::prelude::*;

#[cfg(not(target_os = "windows"))]
#[test]
fn echo_2_times() {
    let mut cmd = Command::cargo_bin("supervise").unwrap();

    cmd.arg("echo abc")
        .arg("-c")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::eq("abc\nabc\n"));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn echo_2_times_with_double_dashes() {
    let mut cmd = Command::cargo_bin("supervise").unwrap();

    cmd.arg("-c")
        .arg("2")
        .arg("--")
        .arg("echo abc")
        .assert()
        .success()
        .stdout(predicate::eq("abc\nabc\n"));
}

#[cfg(target_os = "windows")]
#[test]
fn echo_2_times() {
    let mut cmd = Command::cargo_bin("supervise").unwrap();

    cmd.arg("echo abc")
        .arg("-c")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::eq("abc\r\nabc\r\n"));
}

#[cfg(target_os = "windows")]
#[test]
fn echo_2_times_with_double_dashes() {
    let mut cmd = Command::cargo_bin("supervise").unwrap();

    cmd.arg("-c")
        .arg("2")
        .arg("--")
        .arg("echo abc")
        .assert()
        .success()
        .stdout(predicate::eq("abc\r\nabc\r\n"));
}

#[test]
fn sleep_one_time() {
    let mut cmd = Command::cargo_bin("supervise").unwrap();

    let now = std::time::Instant::now();

    cmd.arg("echo abc")
        .arg("-c")
        .arg("2")
        .arg("-i")
        .arg("0.5")
        .assert()
        .success();

    assert!(now.elapsed() >= std::time::Duration::from_secs_f64(0.5))
}
