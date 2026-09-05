export type QueryClass = 'read' | 'write' | 'empty';

export function classifySql(sql: string): QueryClass {
  const normalized = sql
    .replace(/--.*$/gm, '')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .trim()
    .toLowerCase();
  if (!normalized) return 'empty';
  if (/\b(insert|update|delete|replace|alter|drop|create|attach|detach|vacuum|reindex)\b/.test(normalized)) return 'write';
  if (normalized.startsWith('pragma') && normalized.includes('=')) return 'write';
  const first = normalized.split(/\s+/, 1)[0];
  if (['select', 'with', 'explain', 'pragma'].includes(first)) return 'read';
  return 'write';
}

export function capRows<T>(rows: T[], cap: number): { rows: T[]; truncated: boolean } {
  return { rows: rows.slice(0, cap), truncated: rows.length > cap };
}
