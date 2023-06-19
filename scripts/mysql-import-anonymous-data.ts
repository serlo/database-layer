import { spawn, spawnSync } from 'child_process'
import * as process from 'node:process'

import { IgnoreInsecurePasswordWarning } from './transform'

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
await execSql('source /tmp/mysql.sql;')
console.log('succeeded dump')
await execSql(
  "LOAD DATA LOCAL INFILE '/tmp/user.csv' INTO TABLE user FIELDS TERMINATED BY '\t' LINES TERMINATED BY '\n' IGNORE 1 ROWS;"
)
console.log('succeeded loading')

async function execSql(command: string) {
  await new Promise<void>((resolve, reject) => {
    const dockerComposeExec = spawn('docker-compose', [
      'exec',
      '-T',
      'mysql',
      'sh',
      '-c',
      `mysql --user=root --password="$MYSQL_ROOT_PASSWORD" --local_infile=1 serlo -e "${command}"`,
    ])
    dockerComposeExec.stdout.pipe(process.stdout)
    dockerComposeExec.stderr
      .pipe(new IgnoreInsecurePasswordWarning('utf8'))
      .pipe(process.stderr)
    dockerComposeExec.on('error', (error) => {
      console.error('ERROR: ' + error)
    })
    dockerComposeExec.on('exit', (code) => {
      if (code) {
        reject(code)
      } else {
        resolve()
      }
    })
  })
}
