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
  const semanticVersion = semver.parse(version)

  if (semanticVersion === null) throw new Error(`illegal version ${version}`)

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

  const targetVersions = getTargetVersions(semanticVersion)
  const remoteTags = toTags(remoteName, targetVersions)
  const tags = [...remoteTags, ...toTags(name, targetVersions)]

  spawnSync(
    'docker',
    [
      'build',
      '-f',
      Dockerfile,
      ...R.flatten(tags.map((tag) => ['-t', tag])),
      context,
    ],
    {
      stdio: 'inherit',
    }
  )

  remoteTags.forEach((remoteTag) => {
    console.log('Pushing', remoteTag)
    spawnSync('docker', ['push', remoteTag], { stdio: 'inherit' })
  })
}

function getTargetVersions(version: semver.SemVer) {
  const { major, minor, patch, prerelease } = version

  return prerelease.length > 0
    ? [
        'next',
        ...R.range(0, prerelease.length).map(
          (i) =>
            `${major}.${minor}.${patch}-${prerelease.slice(0, i + 1).join('.')}`
        ),
      ]
    : ['latest', `${major}`, `${major}.${minor}`, `${major}.${minor}.${patch}`]
}

function toTags(name: string, versions: string[]) {
  return versions.map((version) => `${name}:${version}`)
}

interface DockerImageOptions {
  name: string
  version: string
  Dockerfile: string
  context: string
}
