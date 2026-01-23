/** @type {import('jest').Config} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  testMatch: ['**/tests/webdriver/**/*.test.ts'],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json'],
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      tsconfig: 'tsconfig.json'
    }]
  },
  testTimeout: 120000, // 2 minutes default timeout
  maxWorkers: 1, // Run tests sequentially
  bail: false, // Continue running tests even if one fails
  verbose: true,
  collectCoverageFrom: [
    'tests/webdriver/**/*.ts',
    '!tests/webdriver/**/*.test.ts'
  ],
};
