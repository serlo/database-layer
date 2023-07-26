import { Verifier } from '@pact-foundation/pact'
import { spawnSync } from 'node:child_process'
import * as fs from 'node:fs'
import * as path from 'node:path'
import * as process from 'node:process'
import { parse } from 'toml'
import { jest } from '@jest/globals'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

jest.setTimeout(120 * 1000)

test('Pacts', async () => {
  const cargoToml = await fs.promises.readFile(
    path.join(__dirname, '..', 'server', 'Cargo.toml'),
    {
      encoding: 'utf-8',
    },
  )
  const { version } = await parse(cargoToml).package

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
    pactBrokerUsername: process.env.PACT_BROKER_USERNAME ?? 'user',
    pactBrokerPassword: process.env.PACT_BROKER_PASSWORD ?? 'password',
    pactUrls: process.env.PACT_FILE
      ? [path.resolve(process.env.PACT_FILE)]
      : [],
    publishVerificationResult:
      process.env.PUBLISH_VERIFICATION_RESULT === 'true',
    validateSSL: false,
    stateHandlers,
    timeout: 120 * 1000,
    customProviderHeaders: ['Rollback: true'],
  }).verifyProvider()
})
