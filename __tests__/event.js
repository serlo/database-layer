const fetch = require('node-fetch')

const getEventFixtures = require('../__fixtures__/event')

jest.setTimeout(999999999)

const limit = process.env.LIMIT
  ? parseInt(process.env.LIMIT)
  : Number.POSITIVE_INFINITY

test(
  'discussion/comment/archive',
  createTestForEventType('discussion/comment/archive')
)
test(
  'discussion/comment/create',
  createTestForEventType('discussion/comment/create')
)
test('discussion/create', createTestForEventType('discussion/create'))
test('discussion/restore', createTestForEventType('discussion/restore'))
test('entity/create', createTestForEventType('entity/create'))
test('entity/link/create', createTestForEventType('entity/link/create'))
test('entity/link/remove', createTestForEventType('entity/link/remove'))
test('entity/revision/add', createTestForEventType('entity/revision/add'))
test(
  'entity/revision/checkout',
  createTestForEventType('entity/revision/checkout')
)
test('entity/revision/reject', createTestForEventType('entity/revision/reject'))
test('license/object/set', createTestForEventType('license/object/set'))
test(
  'taxonomy/term/associate',
  createTestForEventType('taxonomy/term/associate')
)
test('taxonomy/term/create', createTestForEventType('taxonomy/term/create'))
test(
  'taxonomy/term/dissociate',
  createTestForEventType('taxonomy/term/dissociate')
)
test(
  'taxonomy/term/parent/change',
  createTestForEventType('taxonomy/term/parent/change')
)
test('taxonomy/term/update', createTestForEventType('taxonomy/term/update'))
test('uuid/restore', createTestForEventType('uuid/restore'))
test('uuid/trash', createTestForEventType('uuid/trash'))

function createTestForEventType(name) {
  return async () => {
    let failures = []

    const { values } = getEventFixtures().find((group) => group.name === name)

    for (let i = 0; i < Math.min(limit, values.length); i++) {
      const { id } = values[i]

      console.log(i, id)

      try {
        const aResponse = await fetch(`http://localhost:8080/event/${id}`)
        const bResponse = await fetch(
          `http://de.serlo.localhost:4567/api/event/${id}`
        )

        const a = await aResponse.json()
        const b = await bResponse.json()

        expect(a).toEqual(b)
      } catch (error) {
        failures.push({ name, id, error })
      }
    }

    if (failures.length > 0) {
      console.log(failures)
      throw new Error(`${failures.length} failures.`)
    }
  }
}
