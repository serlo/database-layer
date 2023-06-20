FROM mysql:8.0-debian
RUN apt update && apt install -y pv
COPY docker-entrypoint-initdb.d /docker-entrypoint-initdb.d
COPY conf.d/mysqld.cnf /etc/mysql/conf.d
# The entrypoint script will not run when `serlo.cnf` is in /etc/mysql/conf.d
# as well. Thus we store it in another place and use the `serlo-mysql`
# executable which uses this configuration.
COPY conf.d/serlo.cnf /etc/mysql/
COPY serlo-mysql /usr/local/bin
ENV MYSQL_ROOT_PASSWORD secret
