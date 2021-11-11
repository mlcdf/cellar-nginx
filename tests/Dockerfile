FROM nginx:1.21.3

RUN mkdir /etc/nginx/sites-enabled/ && > /etc/nginx/conf.d/default.conf

COPY tests/fixtures/*.conf /etc/nginx/
COPY target/debug/nvhosts tests/fixtures/nvhosts.toml ./

RUN chmod +x nvhosts && ./nvhosts && cp -r ./sites-available/* /etc/nginx/sites-enabled/

RUN nginx -t
