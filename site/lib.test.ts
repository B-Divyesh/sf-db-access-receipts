import { describe, expect, it } from 'vitest';
import { capRows, classifySql } from './lib';

describe('demo policy', () => {
  it('classifies reads after removing comments', () => {
    expect(classifySql('-- reviewed\nSELECT id FROM orders')).toBe('read');
    expect(classifySql('WITH scoped AS (SELECT 1) SELECT * FROM scoped')).toBe('read');
  });

  it('blocks writes and empty SQL', () => {
    expect(classifySql('DELETE FROM orders')).toBe('write');
    expect(classifySql('WITH scoped AS (SELECT 1) DELETE FROM orders')).toBe('write');
    expect(classifySql('PRAGMA user_version = 2')).toBe('write');
    expect(classifySql('  ')).toBe('empty');
  });

  it('enforces row caps', () => {
    expect(capRows([1, 2, 3], 2)).toEqual({ rows: [1, 2], truncated: true });
  });
});
