# Changelog

All notable changes to this project will be documented in this file.

## [v0.3.51](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.50..v0.3.51) - June 29, 2022

### Added

- Add UserAddRoleMutation #306

- Add UserRemoveRoleMutation #305

## [v0.3.50](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.49..v0.3.50) - June 18, 2022

### Added

- Add optional instance parameter to AllThreadsQuery #304

## [v0.3.49](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.48..v0.3.49) - June 14, 2022

### Added

- Add EntitySortMutation #301

### Changed

- Exercises cannot only be linked to folders and non exercises to topic folders #299

## [v0.3.48](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.47..v0.3.48) - June 8, 2022

### Added

- UserPotentialSpamUsersQuery: filter users with activities and high roles #223

- TaxonomySortMutation works also when not all children ids are given #288

### Fixed

- Fix -2h when adding page revision #284

- PagesQuery: filter pages without revisions #280

- Fix adding of new course page revision with icons #285

## [0.3.47](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.46..v0.3.47) - June 7, 2022

### Fixed

- Fix creation of empty revision of parent of type ExerciseGroup #271

## [0.3.46](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.45..v0.3.46) - May 28, 2022

### Added

- Add PagesQuery #241

## [0.3.45](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.44..v0.3.45) - May 23, 2022

### Added

- Add TaxonomySortMutation #214

- Add friendly error message when storing taxonomy with repeated name

### Fixed

- paginate correctly in DeletedEntitiesQuery #257

## [0.3.44](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.43..v0.3.44) - May 18, 2022

### Added

- Add EntitySetLicenseMutation #249

### Fixed

- TaxonomyTermCreate: query type_id with instance #258

## [0.3.43](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.42..v0.3.43) - May 17, 2022

### Fixed

- return older entries first in DeletedEntitiesQuery #257

## [0.3.42](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.41..v0.3.42) - May 11, 2022

### Added

- Add DeletedEntitiesQuery #240

## [0.3.41](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.40..v0.3.41) - May 9, 2022

### Changed

- Removed LicenseQuery #237

### Fixed

- Ignore `changes` when comparing revisions #238

## [0.3.40](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.39..v0.3.40) - May 6, 2022

### Added

- Add TaxonomyCreateEntityLinkMutation #221

- Add TaxonomyDeleteEntityLinksMutation #222

### Internal

- Remove "instance" from EntityCreateMutation #235

## [0.3.39](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.38..v0.3.39) - May 6, 2022

### Fixed

- Fix autoreview when creating entity #230

- Avoid adding two last not trashed revisions with same content #232

## [0.3.38](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.37..v0.3.38) - May 3, 2022

### Fixed

- Does not checkout entities automatically #220

- Put newly created sub-entities at the end of the list

## [0.3.37](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.36..v0.3.37) - April 26, 2022

### Fixed

- Fix in saving CreateEntityRevisionEvents #218

## [0.3.36](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.35..v0.3.36) - April 22, 2022

### Added

- Add `TaxonomyCreateMutation` #207

- Add `TaxonomyTermMoveMutation`

### Fixed

- Fix 2h delay in saving events #205

## [0.3.35](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.34..v0.3.35) - April 8, 2022

### Added

- Add `AllThreadsQuery`

- Add `UserDeleteRegularUserMutation`

## [0.3.34](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.33..v0.3.34) - March 31, 2022

### Added

- add `TaxonomyTermSetNameAndDescriptionMutation`

- `EntityCreateMutation` accepts taxonomyTermId in payload

## [0.3.33](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.32..v0.3.33) - March 16, 2022

### Added

- add `EntityCreateMutation`

- add `EntityAddRevisionMutation`

- add `PageCreateMutation`

- add `PageAddRevisionMutation`

## [0.3.32](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.31..v0.3.32) - February 3, 2021

### Fixed

- Fix for failing contract tests in LicenseQuery und UserSetEmailMutation

### Internal

- Upgrade to node16

## [0.3.31](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.30..v0.3.31) - January 27, 2022

### Added

- add endpoint `UserSetEmailMutation` (#165)

- add endpoint `UserSetDescriptionMutation` (#166)

### Fixed

- Fix `inLanguage` and `publisher` in metadata api (#167)

## [0.3.30](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.27..v0.3.30) - December 16, 2021

### Added

- Add `EntitiesQuery`.

### Internal

- Update to Rust 1.57.0.

## [0.3.27](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.26..v0.3.27) - November 7, 2021

### Changed

- `UserDeleteBotsMutation` now also returns md5 hashes of removed users.

## [0.3.26](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.25..v0.3.26) - November 1, 2021

### Added

- Add `UserPotentialSpamQuery`.

### Changed

- **`UuidQuery`**. Add `ExerciseGroup.cohesive`

- **`UuidQuery`**. Add `TaxonomyTerm.taxonomyId`

### Fixed

- Creation of threads now counts towards `comments` in `UserActivityByTypeQuery`.

### Internal

- Update to Rust 1.56.0.

- Add support for deploying prereleases.

## [0.3.25](https://github.com/serlo/serlo.org-database-layer/compare/a1d5d8261d84ae546914696c363e92ed83b6a17f..v0.3.25) - October 20, 2021

### Added

- Add `UserDeleteBotsMutation`.
