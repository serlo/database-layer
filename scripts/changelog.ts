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
  const content = generateChangelog({
    repository: {
      firstCommit: 'a1d5d8261d84ae546914696c363e92ed83b6a17f',
      owner: 'serlo',
      repo: 'serlo.org-database-layer',
    },
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
        added: ['Add `UserPotentialSpamQuery`.'],
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
        tagName: 'v0.3.30',
        name: '0.3.30',
        date: '2021-12-16',
        added: ['Add `EntitiesQuery`.'],
        internal: ['Update to Rust 1.57.0.'],
      },
      {
        tagName: 'v0.3.31',
        name: '0.3.31',
        date: '2022-01-27',
        added: [
          'add endpoint `UserSetEmailMutation` (#165)',
          'add endpoint `UserSetDescriptionMutation` (#166)',
        ],
        fixed: ['Fix `inLanguage` and `publisher` in metadata api (#167)'],
      },
      {
        tagName: 'v0.3.32',
        name: '0.3.32',
        date: '2021-02-03',
        internal: ['Upgrade to node16'],
        fixed: [
          'Fix for failing contract tests in LicenseQuery und UserSetEmailMutation',
        ],
      },
      {
        tagName: 'v0.3.33',
        name: '0.3.33',
        date: '2022-03-16',
        added: [
          'add `EntityCreateMutation`',
          'add `EntityAddRevisionMutation`',
          'add `PageCreateMutation`',
          'add `PageAddRevisionMutation`',
        ],
      },
      {
        tagName: 'v0.3.34',
        name: '0.3.34',
        date: '2022-03-31',
        added: [
          'add `TaxonomyTermSetNameAndDescriptionMutation`',
          '`EntityCreateMutation` accepts taxonomyTermId in payload',
        ],
      },
      {
        tagName: 'v0.3.35',
        name: '0.3.35',
        date: '2022-04-08',
        added: ['Add `AllThreadsQuery`', 'Add `UserDeleteRegularUserMutation`'],
      },
      {
        tagName: 'v0.3.36',
        name: '0.3.36',
        date: '2022-04-22',
        added: [
          'Add `TaxonomyCreateMutation` #207',
          'Add `TaxonomyTermMoveMutation`',
        ],
        fixed: ['Fix 2h delay in saving events #205'],
      },
      {
        tagName: 'v0.3.37',
        name: '0.3.37',
        date: '2022-04-26',
        fixed: ['Fix in saving CreateEntityRevisionEvents #218'],
      },
      {
        tagName: 'v0.3.38',
        name: '0.3.38',
        date: '2022-05-03',
        fixed: [
          'Does not checkout entities automatically #220',
          'Put newly created sub-entities at the end of the list',
        ],
      },
      {
        tagName: 'v0.3.39',
        name: '0.3.39',
        date: '2022-05-06',
        fixed: [
          'Fix autoreview when creating entity #230',
          'Avoid adding two last not trashed revisions with same content #232',
        ],
      },
      {
        tagName: 'v0.3.40',
        name: '0.3.40',
        date: '2022-05-06',
        added: [
          'Add TaxonomyCreateEntityLinkMutation #221',
          'Add TaxonomyDeleteEntityLinksMutation #222',
        ],
        internal: ['Remove "instance" from EntityCreateMutation #235'],
      },
      {
        tagName: 'v0.3.41',
        name: '0.3.41',
        date: '2022-05-09',
        fixed: ['Ignore `changes` when comparing revisions #238'],
        changed: ['Removed LicenseQuery #237'],
      },
      {
        tagName: 'v0.3.42',
        name: '0.3.42',
        date: '2022-05-11',
        added: ['Add DeletedEntitiesQuery #240'],
      },
      {
        tagName: 'v0.3.43',
        name: '0.3.43',
        date: '2022-05-17',
        fixed: ['return older entries first in DeletedEntitiesQuery #257'],
      },
      {
        tagName: 'v0.3.44',
        name: '0.3.44',
        date: '2022-05-18',
        added: ['Add EntitySetLicenseMutation #249'],
        fixed: ['TaxonomyTermCreate: query type_id with instance #258'],
      },
      {
        tagName: 'v0.3.45',
        name: '0.3.45',
        date: '2022-05-23',
        added: [
          'Add TaxonomySortMutation #214',
          'Add friendly error message when storing taxonomy with repeated name',
        ],
        fixed: ['paginate correctly in DeletedEntitiesQuery #257'],
      },
      {
        tagName: 'v0.3.46',
        name: '0.3.46',
        date: '2022-05-28',
        added: ['Add PagesQuery #241'],
      },
      {
        tagName: 'v0.3.47',
        name: '0.3.47',
        date: '2022-06-07',
        fixed: [
          'Fix creation of empty revision of parent of type ExerciseGroup #271',
        ],
      },
      {
        tagName: 'v0.3.48',
        date: '2022-06-08',
        added: [
          'UserPotentialSpamUsersQuery: filter users with activities and high roles #223',
          'TaxonomySortMutation works also when not all children ids are given #288',
        ],
        fixed: [
          'Fix -2h when adding page revision #284',
          'PagesQuery: filter pages without revisions #280',
          'Fix adding of new course page revision with icons #285',
        ],
      },
      {
        tagName: 'v0.3.49',
        date: '2022-06-14',
        added: ['Add EntitySortMutation #301'],
        changed: [
          'Exercises cannot only be linked to folders and non exercises to topic folders #299',
        ],
      },
      {
        tagName: 'v0.3.50',
        date: '2022-06-18',
        added: ['Add optional instance parameter to AllThreadsQuery #304'],
      },
      {
        tagName: 'v0.3.51',
        date: '2022-06-29',
        added: [
          'Add UserAddRoleMutation #306',
          'Add UserRemoveRoleMutation #305',
        ],
        fixed: ['Ignore user threads in AllThreadsQuery #310'],
      },
      {
        tagName: 'v0.3.52',
        date: '2022-07-07',
        added: ['add UserCreateMutation #298'],
      },
      {
        tagName: 'v0.3.53',
        date: '2022-07-25',
        added: ['Add UsersByRoleQuery'],
        changed: [
          'Event: throw BadUserInput (not Server Error) in case of inexistent userId',
        ],
      },
      {
        tagName: 'v0.3.54',
        date: '2022-08-24',
        fixed: [
          'Put newly created Taxonomy Terms into the last position in relation to its siblings',
        ],
      },
    ],
  })
  await writeFile(path.join(__dirname, '..', 'CHANGELOG.md'), content)
}
