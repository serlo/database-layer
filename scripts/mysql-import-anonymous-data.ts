import { spawnSync } from 'child_process'

const mysqlCommand = 'mysql --user=root --password="$MYSQL_ROOT_PASSWORD" serlo'
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
spawnSync('gsutil', ['cp', latestDump, `/tmp/${fileName}`], {
  stdio: 'inherit',
})
const container = spawnSync('docker-compose', ['ps', '-q', 'mysql'], {
  stdio: 'pipe',
  encoding: 'utf-8',
})
  .stdout.toString()
  .trim()
spawnSync('unzip', ['-o', `/tmp/${fileName}`, '-d', '/tmp'], {
  stdio: 'inherit',
})
spawnSync('docker', ['cp', '/tmp/mysql.sql', `${container}:/tmp/mysql.sql`], {
  stdio: 'inherit',
})
spawnSync('docker', ['cp', '/tmp/user.csv', `${container}:/tmp/user.csv`], {
  stdio: 'inherit',
})

info('Start importing MySQL data')
execCommand(`pv /tmp/mysql.sql | ${mysqlCommand}`)

info('Start importing anonymized user data')
execSql(
  "LOAD DATA LOCAL INFILE '/tmp/user.csv' INTO TABLE user FIELDS TERMINATED BY '\t' LINES TERMINATED BY '\n' IGNORE 1 ROWS;"
)

function execSql(command: string) {
  execCommand(`${mysqlCommand} --local_infile=1 -e "${command}"`)
}

function execCommand(command: string) {
  const cmd = ['exec', 'mysql', 'sh', '-c', `${command}`]

  spawnSync('docker-compose', cmd, {
    stdio: [process.stdin, process.stdout, process.stderr],
  })
}

function info(message: string) {
  console.error(`INFO: ${message}`)
}
