This directory contains a custom `mysql` docker image. Changes we have made:

- With the command `serlo-mysql` you can use the MySQL client `mysql` together
  with default settings (user, password, database) for accessing the `serlo` DB.
- The command line tool `pv` is available as well. It provides a progress bar
  when a `.sql` file is imported. Usage: `pv file.sql | serlo-mysql`
