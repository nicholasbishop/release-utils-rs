use release_utils::cmd::*;
use std::process::Command;

#[test]
fn test_format_cmd() {
    assert_eq!(format_cmd(&Command::new("echo")), "echo");
    assert_eq!(format_cmd(&Command::new("echo").arg("hello")), "echo hello");
    assert_eq!(
        format_cmd(&Command::new("echo").arg("hello world")),
        "echo hello world"
    );
}
