import { describe, expect, it } from 'vitest';
import { errorMessage, isComposeError } from '../src/types';

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
