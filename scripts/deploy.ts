import { spawnSync } from 'node:child_process'
import * as fs from 'node:fs'
import * as path from 'node:path'
import * as R from 'ramda'
import { parse as semverParse, SemVer } from 'semver'
import * as toml from 'toml'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.join(__dirname, '..')
const cargoTomlPath = path.join(root, 'server', 'Cargo.toml')

const fsOptions: { encoding: BufferEncoding } = { encoding: 'utf-8' }

const { version } = await fetchCargoToml()
buildDockerImage({
  name: 'serlo-org-database-layer',
  version,
  Dockerfile: path.join(root, 'Dockerfile'),
  context: '.',
})

async function fetchCargoToml() {
  const file = await fs.promises.readFile(cargoTomlPath, fsOptions)
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
  const semanticVersion = semverParse(version)

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

function getTargetVersions(version: SemVer) {
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
