use core::time;
use std::any::Any;
use std::env;
use std::net::TcpStream;
use std::ops::Deref;
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::thread;

use lazy_static::lazy_static;

use super::docker;

static START: Once = Once::new();
static STOP: Once = Once::new();
static NB_TESTS: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref RESULTS: Arc<Mutex<Vec<Result<(), Box<dyn Any + Send>>>>> =
        Arc::new(Mutex::new(Vec::<Result<(), Box<dyn Any + Send>>>::new()));
}

fn setup() {
    docker::build(docker::IMAGE_TAG);
    let output = docker::run(docker::IMAGE_TAG);

    let mut counter: u16 = 0;

    loop {
        // faire un read sur docker logs
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

    NB_TESTS.fetch_add(1, Ordering::SeqCst);
    let result = panic::catch_unwind(test);
    let is_success = result.is_ok();
    RESULTS.lock().unwrap().push(result);

    if NB_TESTS.load(Ordering::SeqCst) == RESULTS.lock().unwrap().len() {
        let ok_tests = RESULTS.lock().unwrap();

        let ok_tests: Vec<&Result<(), Box<dyn Any + Send>>> = ok_tests
            .deref()
            .into_iter()
            .filter(|result| result.is_err())
            .collect();

        STOP.call_once(|| {
            teardown(ok_tests.len() > 0);
        });
    }

    assert!(is_success);
}
