import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const appBinary = path.join(root, 'src-tauri', 'target', 'debug', 'app');
const e2eData = path.join(root, '.e2e-data');

export const config = {
  runner: 'local',
  specs: ['./specs/**/*.ts'],
  maxInstances: 1,
  // Keep session capabilities W3C-clean for the embedded WebDriver server.
  // App launch / env live only in the service options below.
  capabilities: [
    {
      browserName: 'tauri',
    },
  ],
  logLevel: 'warn',
  bail: 0,
  waitforTimeout: 15000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 2,
  services: [
    [
      '@wdio/tauri-service',
      {
        appBinaryPath: appBinary,
        driverProvider: 'embedded',
        captureBackendLogs: true,
        env: {
          PROMPT_COMPOSER_E2E: '1',
          XDG_DATA_HOME: e2eData,
        },
      },
    ],
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 120000,
  },
};
