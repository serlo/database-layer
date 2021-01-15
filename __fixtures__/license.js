module.exports = function getLicenseFixtures() {
  if (process.env.DATABASE_ENV === 'staging') {
    // SELECT max(id) as lastId FROM license
    return 19
  }

  return 0
}
