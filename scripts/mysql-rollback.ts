import { spawnSync } from 'node:child_process'

const mysqlRollbackCmd =
  'pv /docker-entrypoint-initdb.d/001-init.sql | serlo-mysql'
const args = ['exec', 'mysql', 'sh', '-c', mysqlRollbackCmd]
const opt = { stdio: [process.stdin, process.stdout, process.stderr] }

spawnSync('docker-compose', args, opt)
