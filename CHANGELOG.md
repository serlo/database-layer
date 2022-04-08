# Changelog

All notable changes to this project will be documented in this file.

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
