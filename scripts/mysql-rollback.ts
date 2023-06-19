import { spawn } from 'node:child_process'
import * as process from 'node:process'

const mysqlRollbackCmd = 'mysql < /docker-entrypoint-initdb.d/001-init.sql'

const dockerComposeArgs = ['exec', '-T', 'mysql', 'sh', '-c', mysqlRollbackCmd]

const sqlRollback = spawn('docker-compose', dockerComposeArgs)

sqlRollback.stdout.pipe(process.stdout)

sqlRollback.stderr.pipe(process.stderr)

sqlRollback.on('error', (error) => {
  console.error('ERROR: ' + error)
})

sqlRollback.on('exit', (code) => {
  process.exit(code !== null ? code : 1)
})
