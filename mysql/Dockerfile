FROM mysql:8.0-debian
RUN apt update && apt install -y pv
COPY conf.d/mysqld.cnf /etc/mysql/conf.d
COPY conf.d/serlo.cnf /etc/mysql/
COPY docker-entrypoint-initdb.d /docker-entrypoint-initdb.d
COPY serlo-mysql /usr/local/bin
ENV MYSQL_ROOT_PASSWORD secret
