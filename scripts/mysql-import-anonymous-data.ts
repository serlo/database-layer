import { spawnSync } from 'child_process'

const latestDump = spawnSync(
  'bash',
  [
    '-c',
    "gsutil ls -l gs://anonymous-data | grep dump | sort -rk 2 | head -n 1 | awk '{ print $3 }'",
  ],
  {
    stdio: 'pipe',
    encoding: 'utf-8',
  }
)
  .stdout.toString()
  .trim()
const fileName = spawnSync('basename', [latestDump], {
  stdio: 'pipe',
  encoding: 'utf-8',
})
  .stdout.toString()
  .trim()

runCmd('gsutil', ['cp', latestDump, `/tmp/${fileName}`])

const container = spawnSync('docker-compose', ['ps', '-q', 'mysql'], {
  stdio: 'pipe',
  encoding: 'utf-8',
})
  .stdout.toString()
  .trim()

runCmd('unzip', ['-o', `/tmp/${fileName}`, '-d', '/tmp'])
runCmd('docker', ['cp', '/tmp/mysql.sql', `${container}:/tmp/mysql.sql`])
runCmd('docker', ['cp', '/tmp/user.csv', `${container}:/tmp/user.csv`])

info('Start importing MySQL data')
execCommand(`pv /tmp/mysql.sql | serlo-mysql`)

info('Start importing anonymized user data')
execSql(
  "LOAD DATA LOCAL INFILE '/tmp/user.csv' INTO TABLE user FIELDS TERMINATED BY '\t' LINES TERMINATED BY '\n' IGNORE 1 ROWS;"
)

function execSql(command: string) {
  execCommand(`serlo-mysql --local_infile=1 -e "${command}"`)
}

function execCommand(command: string) {
  const args = ['exec', 'mysql', 'sh', '-c', `${command}`]

  runCmd('docker-compose', args)
}

function runCmd(cmd: string, args: string[]) {
  const opt = { stdio: [process.stdin, process.stdout, process.stderr] }
  spawnSync(cmd, args, opt)
}

function info(message: string) {
  console.error(`INFO: ${message}`)
}
