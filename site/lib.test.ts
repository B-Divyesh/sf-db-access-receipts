import { describe, expect, it } from 'vitest';
import { cachedLicenseCanUnlock, capRows, classifySql, licenseCacheIsFresh } from './lib';

describe('demo policy', () => {
  it('classifies reads after removing comments', () => {
    expect(classifySql('-- reviewed\nSELECT id FROM orders')).toBe('read');
    expect(classifySql('WITH scoped AS (SELECT 1) SELECT * FROM scoped')).toBe('read');
  });

  it('blocks writes and empty SQL', () => {
    expect(classifySql('DELETE FROM orders')).toBe('write');
    expect(classifySql('  ')).toBe('empty');
  });

  it('enforces row caps', () => {
    expect(capRows([1, 2, 3], 2)).toEqual({ rows: [1, 2], truncated: true });
  });
});

describe('license cache', () => {
  it('unlocks a cached positive verdict while separately tracking freshness', () => {
    const now = 2_000_000_000_000;
    const fresh = JSON.stringify({ valid: true, checkedAt: now - 100 });
    const stale = JSON.stringify({ valid: true, checkedAt: now - 86_400_001 });
    expect(cachedLicenseCanUnlock(fresh)).toBe(true);
    expect(cachedLicenseCanUnlock(stale)).toBe(true);
    expect(cachedLicenseCanUnlock(JSON.stringify({ valid: false, checkedAt: now }))).toBe(false);
    expect(licenseCacheIsFresh(fresh, now)).toBe(true);
    expect(licenseCacheIsFresh(stale, now)).toBe(false);
    expect(cachedLicenseCanUnlock('broken')).toBe(false);
  });
});
