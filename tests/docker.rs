use std::process::{Command, Output};

pub struct RunOutput {
    pub output: Output,
    pub container_id: String,
}

pub fn build(tag: &str) -> Output {
    let output = Command::new("docker")
        .arg("build")
        .arg(".")
        .arg("-f")
        .arg("tests/Dockerfile")
        .arg("--network")
        .arg("host")
        .arg("-t")
        .arg(tag)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        eprintln!(
            "docker build stdout: {:}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "docker build stdout: {:}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}

pub fn run(tag: &str) -> RunOutput {
    let output = Command::new("docker")
        .arg("run")
        .arg("-p")
        .arg("8080:8080")
        .arg("--network")
        .arg("host")
        .arg("-d")
        .arg("-t")
        .arg(tag)
        .output()
        .expect("failed to start container");

    if !output.status.success() {
        eprintln!(
            "docker build stdout: {:}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "docker build stdout: {:}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    RunOutput {
        container_id: String::from_utf8_lossy(&output.stdout).into_owned(),
        output,
    }
}

pub fn clean(id: &String) {
    Command::new("sh")
        .arg("-c")
        .arg(format!("docker stop {}", id))
        .output()
        .expect("failed to stop container");

    Command::new("sh")
        .arg("-c")
        .arg(format!("docker rm {}", id))
        .output()
        .expect("failed to remove container");
}

pub fn logs(id: &String) {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("docker logs {}", id))
        .output()
        .expect("failed to logs container");

    if !output.status.success() {
        eprintln!(
            "docker logs stderr: {:}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    eprintln!("docker logs: {:}", String::from_utf8_lossy(&output.stdout));
}
