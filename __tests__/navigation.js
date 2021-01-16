const fetch = require('node-fetch')

const getInstanceFixtures = require('../__fixtures__/instance')

jest.setTimeout(999999999)

test('navigation', async () => {
    let failures = []

    const instances = getInstanceFixtures()

    for (let instance of instances) {
        try {
            const aResponse = await fetch(`http://localhost:8080/navigation/${instance}`)
            const bResponse = await fetch(
                `http://${instance}.serlo.localhost:4567/api/navigation`
            )

            const a = await aResponse.json()
            const b = await bResponse.json()

            expect(a).toEqual(b)
        } catch (error) {
            failures.push({instance, error})
        }
    }

    if (failures.length > 0) {
        console.log(failures)
        throw new Error(`${failures.length} failures.`)
    }
})
