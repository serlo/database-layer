import { generateChangelog } from '@inyono/changelog'
import * as fs from 'fs'
import * as path from 'path'
import * as util from 'util'

const writeFile = util.promisify(fs.writeFile)

exec()
  .then(() => {
    process.exit(0)
  })
  .catch((error) => {
    console.error(error)
    process.exit(1)
  })

async function exec(): Promise<void> {
  const content = await generateChangelog({
    releases: [
      {
        tagName: 'v0.3.25',
        name: '0.3.25',
        date: '2021-10-20',
        added: ['Add `UserDeleteBotsMutation`.'],
      },
      {
        tagName: 'v0.3.26',
        name: '0.3.26',
        date: '2021-11-01',
        added: [
          'Add `UserPotentialSpamQuery`.',
        ],
        changed: [
          ['`UuidQuery`', 'Add `ExerciseGroup.cohesive`'],
          ['`UuidQuery`', 'Add `TaxonomyTerm.taxonomyId`'],
        ],
        fixed: [
          'Creation of threads now counts towards `comments` in `UserActivityByTypeQuery`.',
        ],
        internal: [
          'Update to Rust 1.56.0.',
          'Add support for deploying prereleases.',
        ],
      },
      {
        tagName: 'v0.3.27',
        name: '0.3.27',
        date: '2021-11-07',
        changed: [
          '`UserDeleteBotsMutation` now also returns md5 hashes of removed users.',
        ],
      },
      {
        tagName: 'v0.3.29',
        name: '0.3.29',
        date: "2021-12-16",
        added: [
            'Add `EntitiesQuery`.'
        ],
        internal: [
          'Update to Rust 1.57.0.',
        ]
      }
    ],
  })
  await writeFile(path.join(__dirname, '..', 'CHANGELOG.md'), content)
}
