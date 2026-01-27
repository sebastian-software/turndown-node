module.exports = {
  testEnvironment: "node",
  testMatch: ["**/parity/**/*.test.js", "**/upstream/**/*.js"],
  testPathIgnorePatterns: ["/node_modules/", "sync-tests.js"],
  verbose: true,
};
