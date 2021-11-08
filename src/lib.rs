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

pub const OUTPUT_DIR: &str = "./sites-available";

pub const TEMPLATE: &str = r#"
server {
    listen      8080;
    listen      [::]:8080;

    location / {
        return 301 https://{{ site.domain }}$request_uri;
    }
}

server {
    listen 8080;
    listen [::]:8080;

    server_name {{ site.domain }};

    include /etc/nginx/security.conf;
    include /etc/nginx/general.conf;

    location / {
        proxy_pass https://cellar-c2.services.clever-cloud.com/{{ site.domain }}/;
        include /etc/nginx/proxy.conf;
    }

    {% for header in site.headers | default(value=[]) %}
    location {{ header.for }} {
        {%- for k, v in header.values %}
        add_header {{ k }} "{{ v }}" always;
        {%- endfor %}
    }
    {% endfor %}

    {% for redirect in site.redirects | default(value=[]) %}
    location = {{ redirect.from }} {
        return {{ redirect.status_code }} {{ redirect.to }};
    }
    {% endfor %}

    {{ site.extra }}
}
"#;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    #[serde(rename = "for")]
    pub for_field: String,
    pub values: HashMap<String, String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Redirect {
    #[serde(rename = "from")]
    pub from_field: String,
    pub to: String,
    pub status_code: u16,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Site {
    pub domain: String,
    pub headers: Option<Vec<Header>>,
    pub redirects: Option<Vec<Redirect>>,
    pub extra: Option<String>,
}

impl Site {
    pub fn generate(
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

    pub fn filename(&self) -> String {
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
            status_code: 302,
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

pub fn redirect_domain(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
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
