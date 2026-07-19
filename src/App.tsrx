import { useEffect, useState } from 'octane';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import {
  archiveStatus,
  clearArchive,
  composeQuery,
  exportArchiveTemplate,
  importArchive,
} from './api';
import type {
  ArchiveStatus,
  ComposeErrorPayload,
  ComposeResult,
} from './types';
import { errorMessage } from './types';

export function App() @{
  const [status, setStatus] = useState(null as ArchiveStatus | null);
  const [bootError, setBootError] = useState(null as string | null);
  const [query, setQuery] = useState('');
  const [result, setResult] = useState(null as ComposeResult | null);
  const [error, setError] = useState(null as ComposeErrorPayload | null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);
  const [showParts, setShowParts] = useState(false);

  useEffect(() => {
    archiveStatus()
      .then((s) => setStatus(s))
      .catch((err) => setBootError(errorMessage(err).message));
  }, []);

  async function onImport() {
    setBusy(true);
    setError(null);
    try {
      const next = await importArchive();
      setStatus(next);
      setResult(null);
    } catch (err) {
      const payload = errorMessage(err);
      if (payload.code !== 'import_cancelled') {
        setError(payload);
      }
    } finally {
      setBusy(false);
    }
  }

  async function onClear() {
    setBusy(true);
    setError(null);
    try {
      const next = await clearArchive();
      setStatus(next);
      setResult(null);
      setShowParts(false);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function onExportTemplate() {
    setBusy(true);
    setError(null);
    try {
      await exportArchiveTemplate();
    } catch (err) {
      const payload = errorMessage(err);
      if (payload.code !== 'export_cancelled') {
        setError(payload);
      }
    } finally {
      setBusy(false);
    }
  }

  async function onCompose() {
    setBusy(true);
    setError(null);
    setCopied(false);
    try {
      const next = await composeQuery(query);
      setResult(next);
    } catch (err) {
      setResult(null);
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  async function onCopy() {
    if (!result) {
      return;
    }
    await writeText(result.prompt);
    setCopied(true);
  }

  const loaded = status?.loaded === true;
  const fileLabel = status?.originalName ?? 'prompt archive';

  <div class="shell" data-testid="app-shell">
    <header class="topbar">
      <div class="brand">
        <h1>
          Prompt <span>Composer</span>
        </h1>
        <p>Archive composer</p>
      </div>

      @if (loaded) {
        <div class="chip" data-testid="archive-chip">
          <span class="dot" />
          <strong data-testid="archive-name">{fileLabel as string}</strong>
          <div class="chip-actions">
            <button
              class="btn ghost"
              data-testid="replace-archive"
              disabled={busy}
              onClick={() => onImport()}
            >
              Replace
            </button>
            <button
              class="btn ghost danger"
              data-testid="clear-archive"
              disabled={busy}
              onClick={() => onClear()}
            >
              Clear
            </button>
          </div>
        </div>
      }
    </header>

    @if (bootError) {
      <div class="errors" data-testid="boot-error">
        <div class="code">boot</div>
        <div class="msg">{bootError as string}</div>
      </div>
    }

    @if (error) {
      <div class="errors" data-testid="error-box">
        <div class="code" data-testid="error-code">{error.code as string}</div>
        <div class="msg" data-testid="error-message">{error.message as string}</div>
        @if (error.suggestion) {
          <div class="sug">{('Suggestion: ' + error.suggestion) as string}</div>
        }
      </div>
    }

    @if (!loaded) {
      <section class="panel empty" data-testid="empty-state">
        <div class="mark">No archive</div>
        <h2>Upload a prompt spreadsheet</h2>
        <p>
          Choose an existing `.xlsx` archive, or create a blank template with the right sheets and
          hint rows to fill in.
        </p>
        <div class="actions center">
          <button
            class="btn primary"
            data-testid="upload-archive"
            disabled={busy}
            onClick={() => onImport()}
          >
            Upload archive
          </button>
          <button
            class="btn"
            data-testid="create-template"
            disabled={busy}
            onClick={() => onExportTemplate()}
          >
            Create blank archive
          </button>
        </div>
      </section>
    } @else {
      <>
        <section class="panel" data-testid="query-panel">
          <div class="label">Query</div>
          <textarea
            class="query"
            data-testid="query-input"
            value={query}
            spellcheck={false}
            onInput={(event) => setQuery(event.currentTarget.value)}
          />
          <p class="hint">
            {'Pattern: '}
            <code>&lt;row&gt; &lt;level&gt;lvl&lt;index&gt; …</code>
            {' — Outfit, Pose, Action, optional Scene'}
          </p>
          <div class="actions">
            <button
              class="btn primary"
              data-testid="compose-button"
              disabled={busy}
              onClick={() => onCompose()}
            >
              Compose
            </button>
          </div>
        </section>

        @if (result) {
          <section
            class={'panel' + (copied ? ' copied' : '')}
            data-testid="output-panel"
          >
            <div class="output-head">
              <div>
                <div class="label">Composed prompt</div>
                <p class="canonical" data-testid="canonical-query">
                  {('Query: ' + result.query) as string}
                </p>
              </div>
              <div class="actions">
                <button class="btn primary" data-testid="copy-button" onClick={() => onCopy()}>
                  {copied ? 'Copied' : 'Copy'}
                </button>
              </div>
            </div>
            <textarea
              class="prompt-out"
              data-testid="prompt-output"
              readonly
              value={result.prompt}
            />
            <button
              class="btn ghost parts-toggle"
              data-testid="toggle-parts"
              onClick={() => setShowParts(!showParts)}
            >
              {showParts ? 'Hide parts' : 'Show parts'}
            </button>
            @if (showParts) {
              <div class="parts" data-testid="parts-list">
                @for (const part of result.parts; key part.kind + part.label) {
                  <div class="part">
                    <div class="kind">{part.label as string}</div>
                    <div class="text">{part.text as string}</div>
                  </div>
                }
              </div>
            }
          </section>
        }
      </>
    }
  </div>
}
