import { spawnSync } from 'child_process'
import * as fs from 'fs'
import * as path from 'path'
import * as R from 'ramda'
import * as semver from 'semver'
import * as toml from 'toml'
import * as util from 'util'

const root = path.join(__dirname, '..')
const cargoTomlPath = path.join(root, 'server', 'Cargo.toml')

const fsOptions: { encoding: BufferEncoding } = { encoding: 'utf-8' }

const readFile = util.promisify(fs.readFile)

void run().then(() => {})

async function run() {
  const { version } = await fetchCargoToml()
  buildDockerImage({
    name: 'serlo-org-database-layer',
    version,
    Dockerfile: path.join(root, 'Dockerfile'),
    context: '.',
  })
}

async function fetchCargoToml() {
  const file = await readFile(cargoTomlPath, fsOptions)
  const { package: pkg } = (await toml.parse(file)) as {
    package: {
      version: string
    }
  }
  return { version: pkg.version }
}

function buildDockerImage({
  name,
  version,
  Dockerfile,
  context,
}: DockerImageOptions) {
  if (!semver.valid(version)) {
    return
  }

  const remoteName = `eu.gcr.io/serlo-shared/${name}`
  const result = spawnSync(
    'gcloud',
    [
      'container',
      'images',
      'list-tags',
      remoteName,
      '--filter',
      `tags=${version}`,
      '--format',
      'json',
    ],
    { stdio: 'pipe' }
  )
  const images = JSON.parse(String(result.stdout)) as unknown[]

  if (images.length > 0) {
    console.log(
      `Skipping deployment: ${remoteName}:${version} already present in registry`
    )
    return
  }

  spawnSync(
    'docker',
    [
      'build',
      '-f',
      Dockerfile,
      ...R.flatten(getTags(version).map((tag) => ['-t', `${name}:${tag}`])),
      context,
    ],
    {
      stdio: 'inherit',
    }
  )

  const remoteTags = R.map((tag) => `${remoteName}:${tag}`, getTags(version))
  remoteTags.forEach((remoteTag) => {
    console.log('Pushing', remoteTag)
    spawnSync('docker', ['tag', `${name}:latest`, remoteTag], {
      stdio: 'inherit',
    })
    spawnSync('docker', ['push', remoteTag], { stdio: 'inherit' })
  })
}

function getTags(version: string) {
  return [
    'latest',
    semver.major(version),
    `${semver.major(version)}.${semver.minor(version)}`,
    `${semver.major(version)}.${semver.minor(version)}.${semver.patch(
      version
    )}`,
  ]
}

interface DockerImageOptions {
  name: string
  version: string
  Dockerfile: string
  context: string
}
