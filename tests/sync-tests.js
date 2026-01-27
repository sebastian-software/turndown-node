#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');

const TURNDOWN_REPO = 'mixmark-io/turndown';
const TURNDOWN_BRANCH = 'master';
const FILES_TO_SYNC = [
  'test/turndown-test.js',
];

const UPSTREAM_DIR = path.join(__dirname, 'upstream');

async function fetchFile(filePath) {
  const url = `https://raw.githubusercontent.com/${TURNDOWN_REPO}/${TURNDOWN_BRANCH}/${filePath}`;

  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => resolve(data));
      res.on('error', reject);
    }).on('error', reject);
  });
}

async function getLatestCommit() {
  const url = `https://api.github.com/repos/${TURNDOWN_REPO}/commits/${TURNDOWN_BRANCH}`;

  return new Promise((resolve, reject) => {
    https.get(url, { headers: { 'User-Agent': 'turndown-node-sync' } }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          const json = JSON.parse(data);
          resolve({
            sha: json.sha.substring(0, 8),
            date: json.commit.committer.date,
            message: json.commit.message.split('\n')[0]
          });
        } catch (e) {
          reject(e);
        }
      });
      res.on('error', reject);
    }).on('error', reject);
  });
}

function adaptTestFile(content, filename) {
  let adapted = content
    // Replace turndown import with turndown-node
    .replace(
      /require\s*\(\s*['"]turndown['"]\s*\)/g,
      "require('turndown-node')"
    )
    .replace(
      /import\s+TurndownService\s+from\s+['"]turndown['"]/g,
      "import TurndownService from 'turndown-node'"
    )
    // Replace any relative test utility imports
    .replace(
      /require\s*\(\s*['"]\.\//g,
      "require('./"
    );

  const header = `/**
 * AUTO-GENERATED - DO NOT EDIT
 *
 * Synchronized from: https://github.com/${TURNDOWN_REPO}/blob/${TURNDOWN_BRANCH}/test/${filename}
 * Run: pnpm sync-tests
 */

`;

  return header + adapted;
}

async function syncTests() {
  console.log('Syncing tests from turndown repository...\n');

  if (!fs.existsSync(UPSTREAM_DIR)) {
    fs.mkdirSync(UPSTREAM_DIR, { recursive: true });
  }

  try {
    const commit = await getLatestCommit();
    console.log(`Latest commit: ${commit.sha} (${commit.date})`);
    console.log(`Message: ${commit.message}\n`);

    for (const filePath of FILES_TO_SYNC) {
      const filename = path.basename(filePath);
      console.log(`Fetching ${filename}...`);

      const content = await fetchFile(filePath);
      const adapted = adaptTestFile(content, filename);
      const destPath = path.join(UPSTREAM_DIR, filename);

      fs.writeFileSync(destPath, adapted);
      console.log(`  -> Saved to ${destPath}`);
    }

    const versionFile = path.join(UPSTREAM_DIR, '.turndown-version');
    fs.writeFileSync(
      versionFile,
      JSON.stringify({ sha: commit.sha, syncedAt: new Date().toISOString() }, null, 2)
    );

    console.log('\nSync complete!');
  } catch (error) {
    console.error('Sync failed:', error.message);
    process.exit(1);
  }
}

syncTests();
