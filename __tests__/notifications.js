const fetch = require('node-fetch')

const getUuidFixtures = require('../__fixtures__/uuid')

jest.setTimeout(999999999)

const limit = process.env.LIMIT
  ? parseInt(process.env.LIMIT)
  : Number.POSITIVE_INFINITY

test('notifications', async () => {
  let failures = []

  const { values } = getUuidFixtures().find(
    (group) => group.discriminator === 'user'
  )

  for (let i = 0; i < Math.min(limit, values.length); i++) {
    const { id } = values[i]
    console.log(i, id)

    try {
      const aResponse = await fetch(`http://localhost:8080/notifications/${id}`)
      const bResponse = await fetch(
        `http://de.serlo.localhost:4567/api/notifications/${id}`
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
