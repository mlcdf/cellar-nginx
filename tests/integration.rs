use core::time;
use std::thread;
use ureq;

mod docker;

const IMAGE_TAG: &str = "nvhosts-test";

#[test]
fn syntax_check() {
    let output = docker::build(IMAGE_TAG);
    assert_eq!(output.status.code().unwrap(), 0)
}

#[test]
fn index_html() {
    docker::build(IMAGE_TAG);
    let output = docker::run(IMAGE_TAG);

    // wait for nginx to start
    thread::sleep(time::Duration::from_secs(3));

    let maybe_response_index = ureq::get("http://0.0.0.0:8080/")
        .set("Host", "dev.www.mlcdf.fr")
        .call();

    let response = match maybe_response_index {
        Ok(response) => {
            response
        }
        Err(err) => {
            docker::logs(&output.container_id);
            docker::clean(&output.container_id);
            panic!("{}", err)
        }
    };

    assert_eq!(response.status(), 200);

    let body = response.into_string().expect("failed to get response body");

    eprintln!("{:?}", body);
    if !body.contains("<!DOCTYPE html>") {
        eprintln!("{:?}", body);
        panic!("response is not a HTML page : body does not contains '<!DOCTYPE html>'");
    }

    if !body.contains("Maxime Le Conte des Floris") {
        eprintln!("{:?}", body);
        panic!("response is not a HTML page : body does not contains 'Maxime Le Conte des Floris");
    }

    let maybe_response_css = ureq::get("http://0.0.0.0:8080/theme/css/poole.css")
        .set("Host", "dev.www.mlcdf.fr")
        .call();

    let response = match maybe_response_css {
        Ok(response) => {
            docker::clean(&output.container_id);
            response
        }
        Err(err) => {
            docker::logs(&output.container_id);
            docker::clean(&output.container_id);
            panic!("{}", err)
        }
    };

    assert_eq!(response.status(), 200);
    assert_eq!(response.header("Content-Type").unwrap(), "text/css; charset=utf-8");
    assert_eq!(response.header("Cache-Control").unwrap(), "public, max-age=31536000, immutable");
}
