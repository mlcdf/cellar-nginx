use core::time;
use std::{env, panic, sync::Once, thread};

use super::docker;

static START: Once = Once::new();

fn setup() {
    docker::build(docker::IMAGE_TAG);
    let output = docker::run(docker::IMAGE_TAG);

    loop {
        match ureq::get("http://0.0.0.0:8080/").call() {
            Ok(_) => break,
            Err(_) => {
                eprintln!("\x1b[0;34mWaiting for container to be ready (10ms)\x1b[0m");
                thread::sleep(time::Duration::from_millis(10))
            }
        }
    }

    env::set_var("CONTAINER_ID", &output.container_id);
}

fn teardown(is_success: bool) {
    let container_id =
        env::var("CONTAINER_ID").expect("No CONTAINER_ID environment variable found.");

    if is_success {
        docker::logs(&container_id);
    }
    docker::clean(&container_id);
}

pub fn run_test<T>(test: T)
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    START.call_once(|| {
        setup();
    });

    let result = panic::catch_unwind(test);

    teardown(result.is_ok());

    assert!(result.is_ok())
}