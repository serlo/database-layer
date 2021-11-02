# Changelog

All notable changes to this project will be documented in this file.

## [0.3.26](https://github.com/serlo/serlo.org-database-layer/compare/v0.3.25..v0.3.26) - November 1, 2021

### Added

- Add query `UserPotentialSpamQuery`

- `UuidQuery`: Add `ExerciseGroup.cohesive`

- `UuidQuery`: Add `TaxonomyTerm.taxonomyId`

### Fixed

- Creation of threads shall count towards `comments` count in `UserActivityByTypeQuery`

### Internal

- Update to new stable rust version (`1.56.0`)

- Begin of refactoring of integration tests

- Add support for deploying prereleases

## [0.3.25](https://github.com/serlo/serlo.org-database-layer/compare/a1d5d8261d84ae546914696c363e92ed83b6a17f..v0.3.25) - October 20, 2021

### Added

- Add mutation `UserDeleteBotsMutation`
