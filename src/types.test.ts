import { describe, expect, it } from 'vitest';
import {
  errorMessage,
  formatCategoryRange,
  formatQueryRangeItems,
  isComposeError,
  type CatalogRanges,
} from '../src/types';

describe('isComposeError', () => {
  it('accepts structured payloads', () => {
    expect(
      isComposeError({
        code: 'no_archive',
        message: 'upload first',
        suggestion: null,
      }),
    ).toBe(true);
  });

  it('rejects unrelated values', () => {
    expect(isComposeError(null)).toBe(false);
    expect(isComposeError('boom')).toBe(false);
    expect(isComposeError({ code: 1, message: 'x' })).toBe(false);
  });
});

describe('errorMessage', () => {
  it('passes through compose errors', () => {
    const payload = {
      code: 'malformed_module',
      message: 'bad token',
      suggestion: 'use NlvlM',
    };
    expect(errorMessage(payload)).toEqual(payload);
  });

  it('wraps Error instances', () => {
    expect(errorMessage(new Error('oops'))).toEqual({
      code: 'unknown',
      message: 'oops',
      suggestion: null,
    });
  });
});

describe('formatQueryRangeItems', () => {
  const ranges: CatalogRanges = {
    subjects: { minRow: 2, maxRow: 3 },
    outfits: { minLevel: 1, maxLevel: 5, minIndex: 1, maxIndex: 30 },
    poses: { minLevel: 1, maxLevel: 3, minIndex: 1, maxIndex: 5 },
    actions: { minLevel: 1, maxLevel: 4, minIndex: 2, maxIndex: 10 },
    scenes: { minLevel: 1, maxLevel: 5, minIndex: 1, maxIndex: 15 },
  };

  it('formats category ranges with spelled-out wording', () => {
    expect(formatCategoryRange(ranges.outfits)).toBe('levels 1–5 · indexes 1–30');
    expect(formatCategoryRange(null)).toBe('(none)');
  });

  it('returns stacked subject and category items', () => {
    expect(formatQueryRangeItems(ranges)).toEqual([
      { label: 'Subjects', value: 'rows 2–3' },
      { label: 'Outfit', value: 'levels 1–5 · indexes 1–30' },
      { label: 'Pose', value: 'levels 1–3 · indexes 1–5' },
      { label: 'Action', value: 'levels 1–4 · indexes 2–10' },
      { label: 'Scene', value: 'levels 1–5 · indexes 1–15' },
    ]);
  });

  it('omits Scene when absent', () => {
    const items = formatQueryRangeItems({ ...ranges, scenes: null });
    expect(items.map((item) => item.label)).toEqual([
      'Subjects',
      'Outfit',
      'Pose',
      'Action',
    ]);
  });
});
