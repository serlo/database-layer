import { spawnSync } from 'child_process'

const TMP_DIR = '/tmp'

main()

function main() {
  const latestDump = getLatestDump()
  if (!latestDump) {
    console.error('❌ Could not fetch latest dump, check your gsutil setup')
    return
  }
  const fileName = getFileName(latestDump)
  downloadDump(latestDump, fileName)

  const container = getMySQLContainer()
  if (!container) {
    console.error(
      '❌ MySQL container not found. Please start the database first with "yarn start"!',
    )
    return
  }

  unzipAndCopyToContainer(fileName, container)
  populateDumpInMySql()
}

function getLatestDump(): string {
  const latestDump = spawnSync(
    'bash',
    [
      '-c',
      "gsutil ls -l gs://anonymous-data | grep dump | sort -rk 2 | head -n 1 | awk '{ print $3 }'",
    ],
    {
      stdio: 'pipe',
      encoding: 'utf-8',
    },
  )
    .stdout.toString()
    .trim()

  return latestDump
}

function getFileName(dumpPath: string): string {
  const fileName = spawnSync('basename', [dumpPath], {
    stdio: 'pipe',
    encoding: 'utf-8',
  })
    .stdout.toString()
    .trim()

  return fileName
}

function downloadDump(dumpPath: string, fileName: string) {
  runCmd('gsutil', ['cp', dumpPath, `${TMP_DIR}/${fileName}`])
}

function getMySQLContainer(): string | null {
  const container = spawnSync('docker', ['compose', 'ps', '-q', 'mysql'], {
    stdio: 'pipe',
    encoding: 'utf-8',
  })
    .stdout.toString()
    .trim()

  return container || null
}

function unzipAndCopyToContainer(fileName: string, container: string) {
  runCmd('unzip', ['-o', `${TMP_DIR}/${fileName}`, '-d', TMP_DIR])
  runCmd('docker', [
    'cp',
    `${TMP_DIR}/mysql.sql`,
    `${container}:/tmp/mysql.sql`,
  ])
  runCmd('docker', ['cp', `${TMP_DIR}/user.csv`, `${container}:/tmp/user.csv`])
}

function populateDumpInMySql() {
  console.log('🟢 Start importing MySQL data')
  execCommand(`pv ${TMP_DIR}/mysql.sql | serlo-mysql`)
  console.log('🟢 Start importing anonymized user data')
  execSql(
    "LOAD DATA LOCAL INFILE '/tmp/user.csv' INTO TABLE user FIELDS TERMINATED BY '\t' LINES TERMINATED BY '\n' IGNORE 1 ROWS;",
  )
}

function execSql(command: string) {
  execCommand(`serlo-mysql --local_infile=1 -e "${command}"`)
}

function execCommand(command: string) {
  const args = ['compose', 'exec', 'mysql', 'sh', '-c', `${command}`]

  runCmd('docker', args)
}

function runCmd(cmd: string, args: string[]) {
  const opt = { stdio: [process.stdin, process.stdout, process.stderr] }
  spawnSync(cmd, args, opt)
}
