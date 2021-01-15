const fetch = require('node-fetch')

jest.setTimeout(999999999)

const limit = process.env.LIMIT
  ? parseInt(process.env.LIMIT)
  : Number.POSITIVE_INFINITY

test('license', async () => {
  let failures = []

  for (let i = 0; i < Math.min(limit, getFixtures()); i++) {
    const id = i + 1
    console.log(id)

    try {
      const aResponse = await fetch(`http://localhost:8080/license/${id}`)
      const bResponse = await fetch(
        `http://de.serlo.localhost:4567/api/license/${id}`
      )

      const a = await aResponse.json()
      const b = await bResponse.json()

      expect(a).toEqual(b)
    } catch (error) {
      failures.push({ id, error })
    }
  }

  if (failures.length > 0) {
    console.log(failures)
    throw new Error(`${failures.length} failures.`)
  }
})

const getFixtures = () => {
  if (process.env.DATABASE_ENV === 'staging') {
    // SELECT max(id) as lastId FROM license
    return 19
  }

  return 0
}
