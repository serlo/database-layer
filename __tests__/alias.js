const fetch = require('node-fetch')

jest.setTimeout(999999999)

test(
  '/user/profile/:username',
  createTestForAlias({
    instance: 'de',
    path: `/user/profile/${
      process.env.DATABASE_ENV === 'staging' ? 'inyono' : 'admin'
    }`,
  })
)
test(
  'Legacy route',
  createTestForAlias({
    instance: 'de',
    path: '/subscribe/1855/true',
  })
)
test(
  'Alias with subject',
  createTestForAlias({
    instance: 'de',
    path: '/mathe/1855/mathe-startseite',
  })
)
test(
  'Alias without subject',
  createTestForAlias({
    instance: 'de',
    path: '/1855/mathe-startseite',
  })
)
test(
  'Alias with empty title',
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
