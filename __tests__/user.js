const fetch = require('node-fetch')

jest.setTimeout(999999999)

test('active-authors', async () => {
  const aResponse = await fetch(`http://localhost:8080/user/active-authors`)
  const bResponse = await fetch(
    `http://de.serlo.localhost:4567/api/user/active-authors`
  )

  const a = await aResponse.json()
  const b = await bResponse.json()

  expect(a).toEqual(b)
})

test('active-reviewers', async () => {
  const aResponse = await fetch(`http://localhost:8080/user/active-reviewers`)
  const bResponse = await fetch(
    `http://de.serlo.localhost:4567/api/user/active-reviewers`
  )

  const a = await aResponse.json()
  const b = await bResponse.json()

  expect(a).toEqual(b)
})
