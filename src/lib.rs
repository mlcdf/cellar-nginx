use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

use tera::{to_value, try_get_value, Context, Tera, Value};

pub mod verbose;

const OUTPUT_DIR: &str = "./sites-available";
const TEMPLATE: &str = include_str!("vhost.template");

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Header {
    #[serde(rename = "for")]
    for_field: String,
    values: HashMap<String, String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Redirect {
    #[serde(rename = "from")]
    from_field: String,
    to: String,
    #[serde(default = "default_redirect_status_code")]
    status_code: u16,
}

fn default_redirect_status_code() -> u16 {
    302
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Site {
    domain: String,
    headers: Option<Vec<Header>>,
    redirects: Option<Vec<Redirect>>,
    extra: Option<String>,
}

impl Site {
    fn generate(
        &self,
        mut tera: MutexGuard<Tera>,
        writer: &mut impl std::io::Write,
    ) -> Result<(), Error> {
        let mut context = Context::new();
        context.insert("site", &self);

        let content = match tera.render_str(TEMPLATE, &context) {
            Ok(x) => x,
            Err(x) => bail!("{:?}", x),
        };

        writer.write(content.as_bytes())?;

        Ok(())
    }

    fn filename(&self) -> String {
        format!("{}.conf", &self.domain)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    sites: Vec<Site>,
}

impl Config {
    pub fn example() -> Self {
        let mut values = HashMap::new();
        values.insert(String::from("Cache-Control"), String::from("public"));
        values.insert(
            String::from("Referrer-Policy"),
            String::from("strict-origin-when-cross-origin"),
        );
        values.insert(
            String::from("Strict-Transport-Security"),
            String::from("max-age=31536000; includeSubDomains; preload"),
        );

        let h = Header {
            for_field: String::from("/*"),
            values: values,
        };

        let r = Redirect {
            from_field: String::from("/example"),
            to: String::from("http://example.com"),
            status_code: 301,
        };

        let example_site = Site {
            domain: String::from("example.com"),
            headers: Some(vec![h]),
            redirects: Some(vec![r]),
            ..Default::default()
        };

        Self {
            sites: vec![example_site],
        }
    }
}

fn redirect_domain(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let mut s = try_get_value!("redirect_domain", "value", String, value);

    s = if s.starts_with("www.") {
        s.replace("www.", "")
    } else {
        format!("www.{}", s)
    };

    Ok(to_value(&s).unwrap())
}

pub fn generate(config: Config) -> Result<(), Error> {
    let tera = Arc::new(Mutex::new(Tera::default()));

    tera.lock()
        .unwrap()
        .register_filter("redirect_domain", redirect_domain);

    fs::create_dir_all(OUTPUT_DIR)?;
    let mut handles = vec![];

    config
        .sites
        .iter()
        .cloned()
        .enumerate()
        .for_each(|(_, site)| {
            let tera = Arc::clone(&tera);

            let handle = thread::spawn(move || {
                let path = Path::new(OUTPUT_DIR).join(site.filename());
                let display = path.display();

                let mut file = match File::create(&path) {
                    Err(why) => bail!("couldn't create {}: {}", display, why),
                    Ok(file) => file,
                };

                site.generate(tera.lock().unwrap(), file.by_ref())?;

                if verbose::is_enabled() {
                    println!("{}", display)
                }

                Ok(())
            });
            handles.push(handle);
        });

    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_redirect_domain() {
        use serde_json::json;

        let value = redirect_domain(&json!("www.mlcdf.fr"), &HashMap::<String, Value>::new());
        assert_eq!(value.unwrap().to_string(), "\"mlcdf.fr\"");

        let value = redirect_domain(&json!("mlcdf.fr"), &HashMap::<String, Value>::new());
        assert_eq!(value.unwrap().to_string(), "\"www.mlcdf.fr\"");

        let value = redirect_domain(&json!("dev.www.mlcdf.fr"), &HashMap::<String, Value>::new());
        assert_eq!(value.unwrap().to_string(), "\"www.dev.www.mlcdf.fr\"");
    }
}
