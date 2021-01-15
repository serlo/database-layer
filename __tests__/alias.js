const fetch = require('node-fetch')

jest.setTimeout(999999999)

test.todo('/user/profile/:username')
test.todo('Legacy route')

test(
  'New Alias with subject',
  createTestForAlias({
    instance: 'de',
    path: '/mathe/1855/mathe-startseite',
  })
)
test(
  'New alias without subject',
  createTestForAlias({
    instance: 'de',
    path: '/1855/mathe-startseite',
  })
)
test(
  'New alias with empty title',
  createTestForAlias({
    instance: 'de',
    path: '/mathe/1855/',
  })
)
test(
  'Legacy Alias',
  createTestForAlias({
    instance: 'de',
    path: '/mathe',
  })
)
test.todo('Legacy Alias with special characters')

function createTestForAlias({ instance, path }) {
  return async () => {
    const aResponse = await fetch(
      `http://localhost:8080/alias/${instance}${path}`
    )
    const bResponse = await fetch(
      `http://${instance}.serlo.localhost:4567/api/alias${path}`
    )

    const a = await aResponse.json()
    const b = await bResponse.json()

    expect(a).toEqual(b)
  }
}
