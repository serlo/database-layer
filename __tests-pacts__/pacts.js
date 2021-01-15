const { Verifier } = require('@pact-foundation/pact')
const { spawnSync } = require('child_process')
const fs = require('fs')
const path = require('path')
const toml = require('toml')
const util = require('util')

jest.setTimeout(120 * 1000)

test('Pacts', async () => {
  async function fetchCargoToml() {
    const readFile = util.promisify(fs.readFile)
    const file = await readFile(path.join(__dirname, '..', 'Cargo.toml'), {
      encoding: 'utf-8',
    })
    const { package: pkg } = await toml.parse(file)
    return { version: pkg.version }
  }

  const { version } = await fetchCargoToml()

  const result = spawnSync('git', ['rev-parse', '--short', 'HEAD'], {
    stdio: 'pipe',
  })
  const hash = String(result.stdout).trim()

  const providerVersion = `${version}-${hash}`

  const handler = {
    get() {
      return () => {
        return Promise.resolve()
      }
    },
  }
  const stateHandlers = new Proxy({}, handler)
  await new Verifier({
    provider: 'serlo.org-database-layer',
    providerVersion,
    providerBaseUrl: 'http://localhost:8080',
    pactBrokerUrl: 'https://pact.serlo.org',
    pactBrokerUsername: process.env.PACT_BROKER_USERNAME,
    pactBrokerPassword: process.env.PACT_BROKER_PASSWORD,
    publishVerificationResult:
      process.env.PUBLISH_VERIFICATION_RESULT === 'true',
    validateSSL: false,
    stateHandlers,
    timeout: 120 * 1000,
  }).verifyProvider()
})
