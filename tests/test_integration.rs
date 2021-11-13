use ureq;

mod common;
use common::{docker, runner};

#[test]
fn syntax_check() {
    let output = docker::build(docker::IMAGE_TAG);
    assert_eq!(output.status.code().unwrap(), 0)
}

#[test]
fn serve_index_html() {
    runner::run_test(|| {
        let maybe_response = ureq::get("http://0.0.0.0:8080/")
            .set("Host", "dev.www.mlcdf.fr")
            .call();

        let response = maybe_response.unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.header("Content-Security-Policy").unwrap(),
            "default-src 'yolo'"
        );

        assert_eq!(
            response.header("X-XSS-Protection").unwrap(),
            "1; mode=block"
        );

        let body = response.into_string().expect("failed to get response body");

        assert!(
            body.contains("<!DOCTYPE html>"),
            "response is not a HTML page : body does not contains '<!DOCTYPE html>'"
        );
        assert!(
            body.contains("Maxime Le Conte des Floris"),
            "response is not a HTML page : body does not contains 'Maxime Le Conte des Floris'"
        );
    });
}

#[test]
fn serve_css() {
    runner::run_test(|| {
        let maybe_response = ureq::get("http://0.0.0.0:8080/theme/css/poole.css")
            .set("Host", "dev.www.mlcdf.fr")
            .call();

        let response = maybe_response.unwrap();

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.header("Content-Type").unwrap(),
            "text/css; charset=utf-8"
        );
        assert_eq!(
            response.header("Cache-Control").unwrap(),
            "public, max-age=31536000, immutable"
        );

        let body = response.into_string().expect("Failed to get body");
        assert!(body.contains("Helvetica"));
    })
}
