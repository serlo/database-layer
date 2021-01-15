const fetch = require('node-fetch')

const getUuidFixtures = require('../__fixtures__/uuid')

jest.setTimeout(999999999)

const limit = process.env.LIMIT
  ? parseInt(process.env.LIMIT)
  : Number.POSITIVE_INFINITY

test('attachment', createTestForDiscriminator('attachment'))
test('blogPost', createTestForDiscriminator('blogPost'))
test('comment', createTestForDiscriminator('comment'))
test('entity', createTestForDiscriminator('entity'))
test('entityRevision', createTestForDiscriminator('entityRevision'))
test('page', createTestForDiscriminator('page'))
test('pageRevision', createTestForDiscriminator('pageRevision'))
test('taxonomyTerm', createTestForDiscriminator('taxonomyTerm'))
test('user', createTestForDiscriminator('user'))

function createTestForDiscriminator(discriminator) {
  return async () => {
    let failures = []

    const { values } = getUuidFixtures().find(
      (group) => group.discriminator === discriminator
    )

    for (let i = 0; i < Math.min(limit, values.length); i++) {
      const { id } = values[i]

      console.log(i, id)

      try {
        const aResponse = await fetch(`http://localhost:8080/uuid/${id}`)
        const bResponse = await fetch(
          `http://de.serlo.localhost:4567/api/uuid/${id}`
        )

        if (aResponse.status !== 200 && bResponse.status !== 200) {
          console.log(id, aResponse.status, bResponse.status)
          continue
        }

        const a = await aResponse.json()
        const b = await bResponse.json()

        // Inconsistency in serlo.org repo. Legacy system incorrectly applies lowercase
        expect({ ...a, alias: a?.alias?.toLowerCase() }).toEqual({
          ...b,
          alias: b?.alias?.toLowerCase(),
        })
      } catch (error) {
        failures.push({ discriminator, id, error })
      }
    }

    if (failures.length > 0) {
      console.log(failures)
      throw new Error(`${failures.length} failures.`)
    }
  }
}
