import { Verifier } from '@pact-foundation/pact'

const targetBranch = 'staging'
const defaultPactFile = `https://raw.githubusercontent.com/serlo/api.serlo.org/${targetBranch}/pacts/api.serlo.org-serlo.org-database-layer.json`

await new Verifier({
  // `localhost` will not work here, see https://stackoverflow.com/a/56731396
  providerBaseUrl: 'http://127.0.0.1:8080',
  customProviderHeaders: ['Rollback: true'],
  pactUrls: [process.env.PACT_FILE ?? defaultPactFile],
}).verifyProvider()
