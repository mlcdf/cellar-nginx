FROM nginx:1.21.3

RUN mkdir /etc/nginx/sites-enabled/

COPY tests/fixtures/*.conf /etc/nginx/
COPY target/debug/nvhosts tests/fixtures/nvhosts.toml ./

RUN ls -l /etc/nginx/
RUN chmod +x nvhosts && ./nvhosts && cp -r ./sites-available/* /etc/nginx/sites-enabled/

RUN nginx -t