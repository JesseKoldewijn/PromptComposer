import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '../..');
const fixturePath = path.join(root, 'fixtures', 'minimal_prompt_archive.xlsx');

const GOLDEN_QUERY = '2 1lvl1 2lvl1 1lvl2';
const GOLDEN_PROMPT = 'BODY_ALPHA OUTFIT_1_1 POSE_2_1 ACTION_1_2';

async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return browser.execute(
    async (command, commandArgs) => {
      const w = window as unknown as {
        __TAURI__?: { core?: { invoke: (c: string, a?: unknown) => Promise<unknown> } };
      };
      const invoke = w.__TAURI__?.core?.invoke;
      if (!invoke) {
        throw new Error('window.__TAURI__.core.invoke not available');
      }
      return invoke(command, commandArgs ?? {});
    },
    cmd,
    args ?? {},
  ) as Promise<T>;
}

describe('Prompt Composer e2e', () => {
  before(async () => {
    expect(fs.existsSync(fixturePath)).toBe(true);
  });

  it('shows empty state on cold start after clear', async () => {
    await tauriInvoke('clear_archive');
    await browser.refresh();
    const empty = await $('[data-testid="empty-state"]');
    await empty.waitForDisplayed({ timeout: 15000 });
  });

  it('imports fixture and composes the golden prompt', async () => {
    await tauriInvoke('import_archive_from_path', { path: fixturePath });
    await browser.refresh();

    const queryPanel = await $('[data-testid="query-panel"]');
    await queryPanel.waitForDisplayed({ timeout: 15000 });

    const chip = await $('[data-testid="archive-chip"]');
    await expect(chip).toBeDisplayed();

    const input = await $('[data-testid="query-input"]');
    await input.waitForDisplayed();
    await input.setValue(GOLDEN_QUERY);

    const compose = await $('[data-testid="compose-button"]');
    await compose.click();

    const output = await $('[data-testid="prompt-output"]');
    await output.waitForDisplayed({ timeout: 10000 });
    await browser.waitUntil(
      async () => (await output.getValue()) === GOLDEN_PROMPT,
      {
        timeout: 10000,
        timeoutMsg: `expected golden prompt, got: ${await output.getValue()}`,
      },
    );
  });

  it('shows validation errors for bad queries', async () => {
    const input = await $('[data-testid="query-input"]');
    await input.setValue('madien 2 1lvl1');
    await (await $('[data-testid="compose-button"]')).click();

    const errorBox = await $('[data-testid="error-box"]');
    await errorBox.waitForDisplayed({ timeout: 10000 });
    const code = await (await $('[data-testid="error-code"]')).getText();
    expect(code).toContain('unknown_keyword');
  });

  it('clears archive back to empty state', async () => {
    const clearBtn = await $('[data-testid="clear-archive"]');
    await clearBtn.click();
    const empty = await $('[data-testid="empty-state"]');
    await empty.waitForDisplayed({ timeout: 10000 });
  });
});
