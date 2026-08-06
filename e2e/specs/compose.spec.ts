import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '../..');
const fixturePath = path.join(root, 'fixtures', 'minimal_prompt_archive.xlsx');

const GOLDEN_QUERY = '2 1lvl1 2lvl1 1lvl2';
const GOLDEN_PROMPT = 'BODY_ALPHA OUTFIT_1_1 POSE_2_1 ACTION_1_2';

interface SubjectRange {
  minRow: number;
  maxRow: number;
}

interface CategoryRange {
  minLevel: number;
  maxLevel: number;
  minIndex: number;
  maxIndex: number;
}

interface CatalogRanges {
  subjects: SubjectRange;
  outfits: CategoryRange | null;
  poses: CategoryRange | null;
  actions: CategoryRange | null;
  scenes: CategoryRange | null;
}

interface ArchiveStatus {
  loaded: boolean;
  ranges: CatalogRanges | null;
}

interface ComposeResult {
  prompt: string;
  query: string;
  parts: { kind: string; label: string; text: string }[];
}

interface ParsedQuery {
  subjectRow: number;
  modules: { level: number; index: number }[];
}

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

function parseQuery(query: string): ParsedQuery {
  const tokens = query.trim().split(/\s+/).filter(Boolean);
  if (tokens.length < 2) {
    throw new Error(`expected row + modules, got: ${query}`);
  }
  const subjectRow = Number(tokens[0]);
  if (!Number.isInteger(subjectRow)) {
    throw new Error(`invalid subject row in query: ${query}`);
  }
  const modules = tokens.slice(1).map((token) => {
    const lvl = /^(\d+)lvl(\d+)$/i.exec(token);
    if (lvl) {
      return { level: Number(lvl[1]), index: Number(lvl[2]) };
    }
    const slash = /^(\d+)\/(\d+)$/.exec(token);
    if (slash) {
      return { level: Number(slash[1]), index: Number(slash[2]) };
    }
    throw new Error(`invalid module token "${token}" in query: ${query}`);
  });
  return { subjectRow, modules };
}

function assertInRange(value: number, min: number, max: number, label: string) {
  expect(value, `${label} ${value} outside ${min}–${max}`).toBeGreaterThanOrEqual(min);
  expect(value, `${label} ${value} outside ${min}–${max}`).toBeLessThanOrEqual(max);
}

function assertQueryWithinRanges(query: string, ranges: CatalogRanges) {
  const parsed = parseQuery(query);
  assertInRange(
    parsed.subjectRow,
    ranges.subjects.minRow,
    ranges.subjects.maxRow,
    'subject row',
  );

  const categoryRanges = [ranges.outfits, ranges.poses, ranges.actions, ranges.scenes];
  expect(parsed.modules.length).toBeGreaterThanOrEqual(3);
  expect(parsed.modules.length).toBeLessThanOrEqual(4);

  for (let i = 0; i < parsed.modules.length; i++) {
    const range = categoryRanges[i];
    expect(range).not.toBeNull();
    const module = parsed.modules[i];
    assertInRange(module.level, range!.minLevel, range!.maxLevel, `module ${i} level`);
    assertInRange(module.index, range!.minIndex, range!.maxIndex, `module ${i} index`);
  }

  if (ranges.scenes) {
    expect(parsed.modules.length).toBe(4);
  } else {
    expect(parsed.modules.length).toBe(3);
  }
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

    const rangeHint = await $('[data-testid="range-hint"]');
    await expect(rangeHint).toBeDisplayed();

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

  it('random fills query within archive sheet ranges and composes', async () => {
    const status = await tauriInvoke<ArchiveStatus>('archive_status');
    expect(status.loaded).toBe(true);
    expect(status.ranges).not.toBeNull();
    const ranges = status.ranges!;

    expect(ranges.subjects.minRow).toBe(2);
    expect(ranges.subjects.maxRow).toBe(3);
    expect(ranges.outfits?.minLevel).toBe(1);
    expect(ranges.outfits?.maxLevel).toBe(5);
    expect(ranges.outfits?.minIndex).toBe(1);
    expect(ranges.outfits?.maxIndex).toBe(30);
    expect(ranges.scenes).not.toBeNull();

    const random = await $('[data-testid="random-button"]');
    await random.waitForDisplayed();

    const input = await $('[data-testid="query-input"]');
    const output = await $('[data-testid="prompt-output"]');
    const canonical = await $('[data-testid="canonical-query"]');

    const seen = new Set<string>();
    for (let i = 0; i < 5; i++) {
      await random.waitForEnabled({ timeout: 10000 });
      await random.click();
      await random.waitForEnabled({ timeout: 10000 });

      await browser.waitUntil(
        async () => {
          const value = await input.getValue();
          return value.trim().split(/\s+/).length === 4;
        },
        {
          timeout: 10000,
          timeoutMsg: `expected randomized query with scene, got: ${await input.getValue()}`,
        },
      );

      const query = await input.getValue();
      assertQueryWithinRanges(query, ranges);
      seen.add(query);

      await output.waitForDisplayed({ timeout: 10000 });
      await browser.waitUntil(
        async () => (await output.getValue()).trim().length > 0,
        {
          timeout: 10000,
          timeoutMsg: 'expected non-empty random prompt',
        },
      );

      const queryText = await canonical.getText();
      expect(queryText).toContain(query);
      expect(await output.getValue()).toContain('BODY_');
    }

    // Backend path: several draws stay inside the same sheet ranges.
    for (let i = 0; i < 10; i++) {
      const result = await tauriInvoke<ComposeResult>('random_compose');
      assertQueryWithinRanges(result.query, ranges);
      expect(result.parts.length).toBe(5);
      expect(result.prompt.trim().length).toBeGreaterThan(0);
      seen.add(result.query);
    }

    expect(seen.size).toBeGreaterThan(1);
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
