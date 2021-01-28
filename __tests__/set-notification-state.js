const fetch = require('node-fetch')

test('set-notification-state', async () => {
  const body = { user_id: 18981, ids: [1565], trashed: true, instance: 'de' }

  const response = await fetch('http://localhost:8080/set-uuid-state', {
    method: 'post',
    body: JSON.stringify(body),
    headers: { 'Content-Type': 'application/json' },
  })
  console.log(response)
  try {
    const data = await response.json()
  } catch (e) {
    console.log(e)
  }
})
