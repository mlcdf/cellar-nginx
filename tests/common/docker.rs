use std::process::{Command, Output};

pub const IMAGE_TAG: &str = "nvhosts-test";

pub struct RunOutput {
    pub output: Output,
    pub container_id: String,
}

/// Builds and tags the docker image
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
        debug(&output, "build");
    }
    output
}

/// Performs a docker run of the given tag
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
        debug(&output, "run");
    }

    RunOutput {
        container_id: String::from_utf8_lossy(&output.stdout).into_owned(),
        output,
    }
}

/// Stops and removes the container with th given id.
///
/// # Panics
///
/// Panics if either the docker stop or docker rm command fails.
pub fn clean(id: &String) {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("docker stop {}", id))
        .output()
        .expect("failed to stop container");

    if !output.status.success() {
        debug(&output, "stop");
        panic!("failed to stop docker container {}", id)
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("docker rm {}", id))
        .output()
        .expect("failed to remove container");

    if !output.status.success() {
        debug(&output, "remove");
        panic!("failed to remove docker container {}", id)
    }
}

/// Prints container logs on stderr.
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

/// Prints the stderr and stdout of a given [`Output`].
fn debug(
    Output {
        status: _,
        stdout,
        stderr,
    }: &Output,
    stage: &str,
) {
    eprintln!(
        "docker {} stdout: {:}",
        stage,
        String::from_utf8_lossy(stdout)
    );
    eprintln!(
        "docker {} stderr: {:}",
        stage,
        String::from_utf8_lossy(stderr)
    );
}
