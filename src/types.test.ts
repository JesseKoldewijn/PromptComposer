import { describe, expect, it } from 'vitest';
import {
  errorMessage,
  formatCategoryRange,
  formatQueryRangeHint,
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

describe('formatQueryRangeHint', () => {
  const ranges: CatalogRanges = {
    subjects: { minRow: 2, maxRow: 3 },
    outfits: { minLevel: 1, maxLevel: 5, minIndex: 1, maxIndex: 30 },
    poses: { minLevel: 1, maxLevel: 3, minIndex: 1, maxIndex: 5 },
    actions: { minLevel: 1, maxLevel: 4, minIndex: 2, maxIndex: 10 },
    scenes: { minLevel: 1, maxLevel: 5, minIndex: 1, maxIndex: 15 },
  };

  it('formats category ranges', () => {
    expect(formatCategoryRange('Outfit', ranges.outfits)).toBe('Outfit: L1–5 / I1–30');
    expect(formatCategoryRange('Pose', null)).toBe('Pose: (none)');
  });

  it('includes subject and category ceilings', () => {
    expect(formatQueryRangeHint(ranges)).toContain('rows 2–3');
    expect(formatQueryRangeHint(ranges)).toContain('Outfit: L1–5 / I1–30');
    expect(formatQueryRangeHint(ranges)).toContain('Scene: L1–5 / I1–15');
  });
});
