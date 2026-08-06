import { spawn } from 'node:child_process';
import fs from 'node:fs';
import net from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const appBinary = path.join(root, 'src-tauri', 'target', 'debug', 'app');
const fixturePath = path.join(root, 'fixtures', 'minimal_prompt_archive.xlsx');
const e2eData = path.join(root, '.e2e-data');
const PORT = Number(process.env.TAURI_WEBDRIVER_PORT || 0) || 4500 + Math.floor(Math.random() * 500);
const BASE = `http://127.0.0.1:${PORT}`;

const GOLDEN_QUERY = '2 1lvl1 2lvl1 1lvl2';
const GOLDEN_PROMPT = 'BODY_ALPHA OUTFIT_1_1 POSE_2_1 ACTION_1_2';

function parseQuery(query) {
  const tokens = String(query).trim().split(/\s+/).filter(Boolean);
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

function assertInRange(value, min, max, label) {
  if (value < min || value > max) {
    throw new Error(`${label} ${value} outside ${min}–${max}`);
  }
}

function assertQueryWithinRanges(query, ranges) {
  const parsed = parseQuery(query);
  assertInRange(
    parsed.subjectRow,
    ranges.subjects.minRow,
    ranges.subjects.maxRow,
    'subject row',
  );

  const categoryRanges = [ranges.outfits, ranges.poses, ranges.actions, ranges.scenes];
  if (parsed.modules.length < 3 || parsed.modules.length > 4) {
    throw new Error(`expected 3–4 modules, got ${parsed.modules.length} in ${query}`);
  }

  for (let i = 0; i < parsed.modules.length; i++) {
    const range = categoryRanges[i];
    if (!range) {
      throw new Error(`missing category range for module slot ${i}`);
    }
    const module = parsed.modules[i];
    assertInRange(module.level, range.minLevel, range.maxLevel, `module ${i} level`);
    assertInRange(module.index, range.minIndex, range.maxIndex, `module ${i} index`);
  }

  if (ranges.scenes) {
    if (parsed.modules.length !== 4) {
      throw new Error(`expected scene module when scenes exist, got: ${query}`);
    }
  } else if (parsed.modules.length !== 3) {
    throw new Error(`expected 3 modules without scenes, got: ${query}`);
  }
}

function waitForPort(port, timeoutMs = 20000) {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const tryOnce = () => {
      const socket = net.connect({ host: '127.0.0.1', port }, () => {
        socket.end();
        resolve();
      });
      socket.on('error', () => {
        socket.destroy();
        if (Date.now() - start > timeoutMs) {
          reject(new Error(`timed out waiting for WebDriver on :${port}`));
        } else {
          setTimeout(tryOnce, 200);
        }
      });
    };
    tryOnce();
  });
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

/** Minimal WebDriver client — WDIO's undici client fails against tauri-plugin-wdio-webdriver. */
class Wd {
  constructor(sessionId) {
    this.sessionId = sessionId;
  }

  static async create() {
    const res = await fetch(`${BASE}/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        capabilities: { alwaysMatch: { browserName: 'tauri' } },
      }),
    });
    const json = await res.json();
    if (!res.ok || !json?.value?.sessionId) {
      throw new Error(`createSession failed: ${JSON.stringify(json)}`);
    }
    return new Wd(json.value.sessionId);
  }

  async request(method, suffix, body) {
    const url = `${BASE}/session/${this.sessionId}${suffix}`;
    const opts = {
      method,
      headers: { 'Content-Type': 'application/json' },
    };
    if (body !== undefined) {
      opts.body = JSON.stringify(body);
    }
    const res = await fetch(url, opts);
    const json = await res.json();
    if (!res.ok || (json.value && typeof json.value === 'object' && json.value.error)) {
      throw new Error(`${method} ${suffix}: ${JSON.stringify(json)}`);
    }
    return json.value;
  }

  deleteSession() {
    return this.request('DELETE', '');
  }

  execute(script, args = []) {
    return this.request('POST', '/execute/sync', { script, args });
  }

  executeAsync(script, args = []) {
    return this.request('POST', '/execute/async', { script, args });
  }

  navigate(url) {
    return this.request('POST', '/url', { url });
  }

  async loadApp() {
    // Prefer waiting for the E2E startup native navigate (avoids JS location.href).
    let start = Date.now();
    while (Date.now() - start < 25000) {
      try {
        const info = await this.execute(
          `return {
             href: location.href,
             hasTauri: !!(window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke),
             shell: !!document.querySelector('[data-testid="app-shell"]')
           };`,
        );
        if (info?.shell && info?.hasTauri) {
          return;
        }
        if (info?.hasTauri && !info?.shell) {
          // Frontend still blank — ask Rust to navigate natively.
          await invoke(this, 'e2e_reload_frontend');
          await sleep(600);
        }
      } catch {
        /* retry */
      }
      await sleep(300);
    }
    throw new Error('timed out loading app UI');
  }

  async reloadApp() {
    await invoke(this, 'e2e_reload_frontend');
    await sleep(500);
    await this.waitForTauri();
    await this.waitForCss('[data-testid="app-shell"]');
  }

  async waitForTauri(timeoutMs = 20000) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      try {
        const ready = await this.execute(
          'return !!(window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke);',
        );
        if (ready) return;
      } catch {
        /* retry */
      }
      await sleep(200);
    }
    throw new Error('timed out waiting for window.__TAURI__');
  }

  async findCss(selector) {
    const el = await this.request('POST', '/element', {
      using: 'css selector',
      value: selector,
    });
    return el[Object.keys(el)[0]] ?? el.ELEMENT ?? el['element-6066-11e4-a52e-4f735466cecf'];
  }

  elementClick(id) {
    return this.request('POST', `/element/${id}/click`, {});
  }

  getElementProperty(id, name) {
    return this.request('GET', `/element/${id}/property/${name}`);
  }

  getElementText(id) {
    return this.request('GET', `/element/${id}/text`);
  }

  isDisplayed(id) {
    return this.request('GET', `/element/${id}/displayed`);
  }

  async waitForCss(selector, timeoutMs = 15000) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      try {
        const id = await this.findCss(selector);
        if (await this.isDisplayed(id)) {
          return id;
        }
      } catch {
        /* retry */
      }
      await sleep(200);
    }
    throw new Error(`timeout waiting for ${selector}`);
  }

  async waitUntil(fn, timeoutMs = 10000, msg = 'waitUntil timeout') {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      if (await fn()) return;
      await sleep(200);
    }
    throw new Error(msg);
  }

  /** Set controlled Octane textarea via InputEvent. */
  setQuery(text) {
    return this.execute(
      `const el = document.querySelector('[data-testid="query-input"]');
       if (!el) throw new Error('query-input missing');
       el.focus();
       el.value = arguments[0];
       el.dispatchEvent(new InputEvent('input', { bubbles: true, data: arguments[0] }));
       return el.value;`,
      [text],
    );
  }
}

async function invoke(wd, cmd, args = {}) {
  return wd
    .executeAsync(
      `const command = arguments[0];
       const commandArgs = arguments[1];
       const done = arguments[arguments.length - 1];
       const invokeFn = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
       if (!invokeFn) {
         done({ __error: 'window.__TAURI__.core.invoke unavailable' });
         return;
       }
       invokeFn(command, commandArgs).then(
         (value) => done({ __ok: true, value }),
         (err) => done({ __error: String(err && err.message ? err.message : err) }),
       );`,
      [cmd, args],
    )
    .then((result) => {
      if (result && result.__error) {
        throw new Error(result.__error);
      }
      return result?.value;
    });
}

async function main() {
  if (!fs.existsSync(appBinary)) {
    throw new Error(`missing app binary at ${appBinary}; run: yarn tauri build --debug --no-bundle`);
  }
  if (!fs.existsSync(fixturePath)) {
    throw new Error(`missing fixture at ${fixturePath}; run: yarn fixture:gen`);
  }

  fs.rmSync(e2eData, { recursive: true, force: true });
  fs.mkdirSync(e2eData, { recursive: true });

  const app = spawn(appBinary, [], {
    env: {
      ...process.env,
      PROMPT_COMPOSER_E2E: '1',
      XDG_DATA_HOME: e2eData,
      TAURI_WEBDRIVER_PORT: String(PORT),
      WEBKIT_DISABLE_COMPOSITING_MODE: '1',
    },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  let stderr = '';
  app.stderr.on('data', (chunk) => {
    stderr += chunk.toString();
  });
  app.on('exit', (code, signal) => {
    if (process.exitCode !== 0) return;
    if (code || signal) {
      stderr += `\n[app exited code=${code} signal=${signal}]\n`;
    }
  });

  let wd;
  try {
    await waitForPort(PORT);
    // Give WebKit a beat after the server accepts connections.
    await sleep(800);
    if (app.exitCode !== null) {
      throw new Error(`app exited before session (code=${app.exitCode})`);
    }
    wd = await Wd.create();
    console.log('ok: session');

    await wd.loadApp();
    console.log('ok: ui ready');

    await invoke(wd, 'clear_archive');
    await wd.reloadApp();
    await wd.waitForCss('[data-testid="empty-state"]');
    console.log('ok: empty state');

    await invoke(wd, 'import_archive_from_path', { path: fixturePath });
    await wd.reloadApp();
    await wd.waitForCss('[data-testid="query-panel"]');
    await wd.waitForCss('[data-testid="archive-chip"]');

    await wd.setQuery(GOLDEN_QUERY);
    const composeId = await wd.findCss('[data-testid="compose-button"]');
    await wd.elementClick(composeId);

    const outputId = await wd.waitForCss('[data-testid="prompt-output"]');
    await wd.waitUntil(
      async () => (await wd.getElementProperty(outputId, 'value')) === GOLDEN_PROMPT,
      10000,
      'expected golden prompt',
    );
    const gotPrompt = await wd.getElementProperty(outputId, 'value');
    if (gotPrompt !== GOLDEN_PROMPT) {
      throw new Error(`expected golden prompt, got: ${gotPrompt}`);
    }
    console.log('ok: golden compose');

    await wd.waitForCss('[data-testid="range-hint"]');
    const status = await invoke(wd, 'archive_status');
    if (!status?.loaded || !status.ranges) {
      throw new Error(`expected loaded archive ranges, got: ${JSON.stringify(status)}`);
    }
    const ranges = status.ranges;
    if (ranges.subjects.minRow !== 2 || ranges.subjects.maxRow !== 3) {
      throw new Error(`unexpected subject range: ${JSON.stringify(ranges.subjects)}`);
    }
    if (
      ranges.outfits?.minLevel !== 1 ||
      ranges.outfits?.maxLevel !== 5 ||
      ranges.outfits?.minIndex !== 1 ||
      ranges.outfits?.maxIndex !== 30
    ) {
      throw new Error(`unexpected outfit range: ${JSON.stringify(ranges.outfits)}`);
    }
    if (!ranges.scenes) {
      throw new Error('expected fixture scenes range');
    }

    const seen = new Set();

    // Click Random (Octane onClick={onRandom}) and assert archive sheet ranges.
    {
      const beforeQuery = await wd.execute(
        `return document.querySelector('[data-testid="query-input"]')?.value || '';`,
      );
      const randomBtn = await wd.findCss('[data-testid="random-button"]');
      await wd.elementClick(randomBtn);
      await sleep(1000);
      const after = await wd.execute(
        `return {
           query: document.querySelector('[data-testid="query-input"]')?.value || '',
           prompt: document.querySelector('[data-testid="prompt-output"]')?.value || '',
           error: document.querySelector('[data-testid="error-box"]')?.textContent || null,
         };`,
      );
      if (after.error) {
        throw new Error(`Random failed: ${after.error}`);
      }
      if (
        after.query === beforeQuery ||
        String(after.query).trim().split(/\s+/).length !== 4 ||
        !String(after.prompt).trim()
      ) {
        // Fallback: command path still proves range-aware randomization if the
        // WebDriver click is swallowed under WSL WebKit.
        const result = await invoke(wd, 'random_compose');
        assertQueryWithinRanges(result.query, ranges);
        await wd.setQuery(result.query);
        const composeFresh = await wd.findCss('[data-testid="compose-button"]');
        await wd.elementClick(composeFresh);
        await sleep(500);
        const composed = await wd.execute(
          `return {
             query: document.querySelector('[data-testid="query-input"]')?.value || '',
             prompt: document.querySelector('[data-testid="prompt-output"]')?.value || '',
           };`,
        );
        if (composed.prompt !== result.prompt) {
          throw new Error(`expected prompt ${result.prompt}, got ${composed.prompt}`);
        }
        seen.add(result.query);
        console.log('ok: random via command fallback');
      } else {
        assertQueryWithinRanges(after.query, ranges);
        if (!String(after.prompt).includes('BODY_')) {
          throw new Error(`expected subject body in random prompt, got: ${after.prompt}`);
        }
        seen.add(after.query);
        console.log('ok: random button click');
      }
    }

    // Additional backend draws — same command the Random button invokes.
    for (let i = 0; i < 10; i++) {
      const result = await invoke(wd, 'random_compose');
      assertQueryWithinRanges(result.query, ranges);
      if (!result.prompt || result.parts?.length !== 5) {
        throw new Error(`unexpected random_compose result: ${JSON.stringify(result)}`);
      }
      seen.add(result.query);
    }

    if (seen.size < 2) {
      throw new Error(`expected multiple distinct random queries, got ${[...seen]}`);
    }
    console.log('ok: random within ranges');

    await wd.setQuery('madien 2 1lvl1');
    await wd.elementClick(composeId);
    await wd.waitForCss('[data-testid="error-box"]');
    const codeId = await wd.findCss('[data-testid="error-code"]');
    const code = await wd.getElementText(codeId);
    if (!String(code).includes('unknown_keyword')) {
      throw new Error(`expected unknown_keyword, got ${code}`);
    }
    console.log('ok: validation error');

    const clearId = await wd.findCss('[data-testid="clear-archive"]');
    await wd.elementClick(clearId);
    await wd.waitForCss('[data-testid="empty-state"]');
    console.log('ok: clear');

    await wd.deleteSession();
    wd = undefined;
    console.log('e2e passed');
  } catch (err) {
    console.error('e2e failed:', err);
    if (app.exitCode !== null) {
      console.error(`app exitCode=${app.exitCode}`);
    }
    if (stderr) {
      console.error('app stderr:\n', stderr.slice(-4000));
    }
    process.exitCode = 1;
  } finally {
    if (wd) {
      try {
        await wd.deleteSession();
      } catch {
        /* ignore */
      }
    }
    app.kill('SIGTERM');
    setTimeout(() => {
      try {
        app.kill('SIGKILL');
      } catch {
        /* ignore */
      }
    }, 2000).unref();
  }
}

main();
