import { generateChangelog } from '@inyono/changelog'
import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
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
    {
      tagName: 'v0.3.55',
      date: '2022-10-05',
      fixed: [
        'Update payload of `UserDeleteRegularUserMutation` to match API contract #327',
        'Do not accept user descriptions bigger than 64KB #323',
        'Fix alias for page revisions #325',
      ],
    },
    {
      tagName: 'v0.3.56',
      date: '2022-11-02',
      added: ['Order AllThreadsQuery according to last updated'],
    },
    {
      tagName: 'v0.3.57',
      date: '2022-11-22',

      fixed: ['Fix sorting of threads in discussions #345'],
    },

    {
      tagName: 'v0.3.58',
      date: '2023-01-04',

      fixed: ['Really trash or restore taxonomy term #350'],
    },
    {
      tagName: 'v0.3.59',
      date: '2023-01-26',
      added: [
        'Add ThreadEditMutation, Query also if user subscribed for email and if it was already sent',
      ],
      fixed: ['Do not query checkout event of page revision'],
    },
    {
      tagName: 'v0.3.60',
      date: '2023-02-02',
      fixed: ['thread: fix name of ThreadEditMutation'],
    },
    {
      tagName: 'v0.3.61',
      date: '2023-03-02',
      added: ['Add serlo.org/268835 "Chancenwerk" as subject'],
    },
    {
      tagName: 'v0.3.62',
      date: '2023-03-02',
      fixed: ['Make all taxonomies under "Mathematik > Partner" as subjects'],
    },
    {
      tagName: 'v0.3.63',
      date: '2023-03-06',
      fixed: ['Make all taxonomies under "Fächer im Aufbau" subjects'],
    },
    {
      tagName: 'v0.3.64',
      date: '2023-05-09',
      added: [
        'metadata: add fields mainentityofpage, interactivityType and isPartOf; compute better name for exercises, and Update `learningResourceType`',
      ],
      fixed: [
        'metadata: comply to amb regarding field description, change `@context` to that it complies with schema.org, fix `type` for applets',
        'entity: prevent creation of two solutions for the same exercise',
        'thread: handle state change of many comments, not only of the first one',
      ],
    },
    {
      tagName: 'v0.3.65',
      date: '2023-06-19',
      added: [
        'add about to MetadataQuery',
        'filter threads by subjectId',
        'Metadata: Return dict for version',
        'Ignore Metadata Query param modifiedAfter if API has changed recently',
      ],
      changed: [
        'Change `mainEntityOfPage` to an array',
        'Update organization link to /organization',
        'Update URL in `@context`',
        'remove headline when empty',
        'Always use the same JSON for Serlo',
        'distinguish Serlo as organization from website',
      ],
    },
    {
      tagName: 'v0.3.66',
      date: '2023-06-20',
      fixed: ['Fix metadata query 500 when hitting entity 1613'],
    },
    {
      tagName: 'v0.3.67',
      date: '2023-07-26',
      fixed: [
        'Metadata API: Adjust subject mapping',
        'Metadata API: Add "type" to "mainEntityOfPage"',
      ],
      internal: [
        'Upgrade sqlx to 0.7.1',
        'Upgrade various dependencies',
        'Refactor to use Serlo MySQL database from serlo-mysql-database Docker image',
        'Setup dependabot for dependencies upgrades',
      ],
    },
    {
      tagName: 'v0.3.68',
      date: '2023-08-23',
      added: [
        'metadata-api: Add new learning resource type of WLO, metadata-api: Change WebAPI to WebContent',
      ],
    },
    {
      tagName: 'v0.3.69',
      date: '2023-08-24',
      changed: ['database base: change character encoding'],
    },
    {
      tagName: 'v0.3.70',
      date: '2023-09-04',
      added: [
        'threads: query property status',
        'Add ThreadSetThreadStatusMutation',
        'Add param status to AllThreadsQuery',
      ],
      internal: ['New DB dump to add `status` to `comment`'],
    },
  ],
})

await fs.promises.writeFile(path.join(__dirname, '..', 'CHANGELOG.md'), content)
