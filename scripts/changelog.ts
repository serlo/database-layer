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
        added: ['Add mutation `UserDeleteBotsMutation`'],
      },
      {
        tagName: 'v0.3.26',
        name: '0.3.26',
        date: '2021-11-01',
        added: [
          'Add query `UserPotentialSpamQuery`',
          '`UuidQuery`: Add `ExerciseGroup.cohesive`',
          '`UuidQuery`: Add `TaxonomyTerm.taxonomyId`',
        ],
        fixed: [
          'Creation of threads shall count towards `comments` count in `UserActivityByTypeQuery`',
        ],
        internal: [
          'Update to new stable rust version (`1.56.0`)',
          'Begin of refactoring of integration tests',
          'Add support for deploying prereleases',
        ],
      },
      {
        tagName: 'v0.3.27',
        name: '0.3.27',
        date: '2021-11-07',
        added: [
          '`UserDeleteBotsMutation` also returns md5 hashes of removed user',
        ],
      },
    ],
  })
  await writeFile(path.join(__dirname, '..', 'CHANGELOG.md'), content)
}
