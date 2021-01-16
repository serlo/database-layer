module.exports = function getInstanceFixtures() {
    if (process.env.DATABASE_ENV === 'staging') {
        return ['de', 'en', 'es', 'fr', 'hi', 'ta']
    }

    return ['de', 'en']
}
