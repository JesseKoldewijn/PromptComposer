import { invoke } from '@tauri-apps/api/core';
import type { ArchiveStatus, ComposeResult } from './types';

export function archiveStatus() {
  return invoke<ArchiveStatus>('archive_status');
}

export function importArchive() {
  return invoke<ArchiveStatus>('import_archive');
}

export function clearArchive() {
  return invoke<ArchiveStatus>('clear_archive');
}

export function composeQuery(query: string) {
  return invoke<ComposeResult>('compose_query', { query });
}

export function exportArchiveTemplate() {
  return invoke<string>('export_archive_template');
}
