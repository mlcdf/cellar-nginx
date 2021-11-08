use std::process::Command;

#[test]
fn syntax_check() {
    let output = Command::new("docker")
        .arg("build")
        .arg(".")
        .arg("-f")
        .arg("tests/Dockerfile")
        .arg("-t")
        .arg("nvhosts-test")
        .output()
        .expect("failed to execute process");

    eprintln!("stdout: {:}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {:}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.status.code().unwrap(), 0)
}
