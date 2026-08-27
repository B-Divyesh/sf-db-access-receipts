export type QueryClass = 'read' | 'write' | 'empty';

export function classifySql(sql: string): QueryClass {
  const normalized = sql
    .replace(/--.*$/gm, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .trim()
    .toLowerCase();
  if (!normalized) return 'empty';
  const first = normalized.split(/\s+/, 1)[0];
  if (['select', 'with', 'explain', 'pragma'].includes(first)) return 'read';
  return 'write';
}

export function cachedLicenseCanUnlock(
  raw: string | null,
  now = Date.now(),
): boolean {
  if (!raw) return false;
  try {
    const parsed = JSON.parse(raw) as { valid?: unknown; checkedAt?: unknown };
    return (
      parsed.valid === true &&
      typeof parsed.checkedAt === 'number' &&
      now - parsed.checkedAt < 86_400_000
    );
  } catch {
    return false;
  }
}

export function capRows<T>(rows: T[], cap: number): { rows: T[]; truncated: boolean } {
  return { rows: rows.slice(0, cap), truncated: rows.length > cap };
}
