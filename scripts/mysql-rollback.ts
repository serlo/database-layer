import { spawn } from 'node:child_process'
import * as process from 'node:process'
import { IgnoreInsecurePasswordWarning } from './transform'

const mysqlRollbackCmd =
  'mysql --user=root --password="$MYSQL_ROOT_PASSWORD"' +
  ' < /docker-entrypoint-initdb.d/001-init.sql'

const dockerComposeArgs = ['exec', '-T', 'mysql', 'sh', '-c', mysqlRollbackCmd]

const sqlRollback = spawn('docker-compose', dockerComposeArgs)

sqlRollback.stdout.pipe(process.stdout)

sqlRollback.stderr
  .pipe(new IgnoreInsecurePasswordWarning('utf8'))
  .pipe(process.stderr)

sqlRollback.on('error', (error) => {
  console.error('ERROR: ' + error)
})

sqlRollback.on('exit', (code) => {
  process.exit(code !== null ? code : 1)
})
