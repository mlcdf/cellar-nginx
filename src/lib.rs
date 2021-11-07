use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use tera::{Context, Tera, try_get_value, to_value, Value};

pub const OUTPUT_DIR: &str = "./sites-available";

pub const TEMPLATE: &str = r#"
server {
    listen      80;
    listen      [::]:80;
    server_name {{ site.domain | redirect_domain }};

    location / {
        return 301 https://{{ site.domain }}$request_uri;
    }
}

server {
    listen 80;
    listen [::]:80;

    server_name {{ site.domain }};

    include /etc/nginx/security.conf;
    include /etc/nginx/general.conf;

    location / {
        proxy_pass https://cellar-c2.services.clever-cloud.com/{{ site.domain }}/;
        include /etc/nginx/proxy.conf;
    }

    {% for header in site.headers %}
    location {{ header.for }} {
        {%- for k, v in header.values %}
        add_header {{ k }} "{{ v }}" always;
        {%- endfor %}
    }
    {% endfor %}
}
"#;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Header {
    #[serde(rename = "for")]
    pub for_field: String,
    pub values: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Site {
    pub domain: String,
    pub headers: Vec<Header>,
}

impl Site {
    pub fn generate(&self, tera: Rc<RefCell<Tera>>, writer: &mut impl std::io::Write) -> Result<(), Error> {
        let mut context = Context::new();
        context.insert("site", &self);

        let content = tera.borrow_mut().render_str(TEMPLATE, &context)?;
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

        let example_site = Site {
            domain: String::from("example.com"),
            headers: vec![h],
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


pub fn generate(config: &Config) -> Result<(), Error> {
    let tera = Rc::new(RefCell::new(Tera::default()));
    tera.borrow_mut().register_filter("redirect_domain", redirect_domain);

    fs::create_dir_all(OUTPUT_DIR)?;

    for (_, site) in config.sites.iter().enumerate() {
        let path = Path::new(OUTPUT_DIR).join(site.filename());
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        site.generate(Rc::clone(&tera), file.by_ref())?;
    }

    Ok(())
}
