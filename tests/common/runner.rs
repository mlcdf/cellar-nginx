use core::time;
use std::env;
use std::net::TcpStream;
use std::panic;
use std::sync::Once;
use std::thread;

use super::docker;

static START: Once = Once::new();

fn setup() {
    docker::build(docker::IMAGE_TAG);
    let output = docker::run(docker::IMAGE_TAG);

    let mut counter: u16 = 0;

    loop {
        match TcpStream::connect("0.0.0.0:8080") {
            Ok(_) => break,
            Err(err) => {
                if counter == 100 {
                    docker::logs(&output.container_id);
                    eprintln!("{:}", err);
                    panic!(
                        "\x1b[0;31mFailed to start container in less than {}ms\x1b[0m",
                        10 * counter
                    )
                }
                thread::sleep(time::Duration::from_millis(10));
                counter += 1;
                eprint!(
                    "\x1b[0;34mWaiting for container to be ready ({}ms)\x1b[0m\x1B[0K\r",
                    10 * counter
                );
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
