# Claude Code Notes

## npm Publishing with Trusted Publishers (OIDC)

**Important:** npm OIDC/Trusted Publishers requires **npm v11+**, which comes with **Node.js 24**.

Node.js 22 ships with npm v10, which does NOT support OIDC token exchange for Trusted Publishers.

### Working Configuration

```yaml
permissions:
  id-token: write # Required for OIDC

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: "24" # npm v11+ required for OIDC
          registry-url: "https://registry.npmjs.org"

      - run: npm publish --access public --provenance
        # No NODE_AUTH_TOKEN needed - OIDC handles auth
```

### npm Package Setup

Each package must have Trusted Publishers configured on npmjs.com:

- Settings → Publishing access → Add GitHub Actions
- Repository: `owner/repo`
- Workflow filename: `publish.yml` (must match exactly)
- Environment: (leave empty unless using GitHub environments)
