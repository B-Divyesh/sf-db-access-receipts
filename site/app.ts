import './styles.css';
import { classifySql } from './lib';

const isDemo = document.body.dataset.demo === 'true';
const receiptsKey = isDemo ? 'demo:db-receipts:receipts' : 'db-receipts:receipts';
const themeKey = isDemo ? 'demo:db-receipts:theme' : 'db-receipts:theme';
const challenge = 'FERN-42';

type DemoReceipt = {
  id: string;
  time: string;
  actor: string;
  kind: 'template' | 'novel';
  queryHash: string;
  outcome: 'allowed' | 'denied';
  approval: string;
  rows: number;
  rowCap: number;
  columnCap: number;
  reason: string;
};

const required = <T extends Element>(selector: string): T => {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`Missing element: ${selector}`);
  return element;
};

const modeButtons = [...document.querySelectorAll<HTMLButtonElement>('[data-mode]')];
const form = required<HTMLFormElement>('#query-form');
const sqlInput = required<HTMLTextAreaElement>('#sql');
const actorInput = required<HTMLInputElement>('#actor');
const accountInput = required<HTMLInputElement>('#account-id');
const approvalGroup = required<HTMLElement>('#approval-group');
const approvalInput = required<HTMLInputElement>('#approval');
const modeDescription = required<HTMLElement>('#mode-description');
const submitButton = required<HTMLButtonElement>('#run-query');
const receiptSheet = required<HTMLElement>('#receipt-sheet');
const emptyReceipt = required<HTMLElement>('#receipt-empty');
const errorMessage = required<HTMLElement>('#form-error');
const historyList = required<HTMLOListElement>('#receipt-history');
const clearButton = required<HTMLButtonElement>('#clear-history');
let activeMode: 'template' | 'novel' = 'template';

function setMode(mode: 'template' | 'novel'): void {
  activeMode = mode;
  modeButtons.forEach((button) => {
    const selected = button.dataset.mode === mode;
    button.setAttribute('aria-selected', String(selected));
    button.tabIndex = selected ? 0 : -1;
  });
  const novel = mode === 'novel';
  sqlInput.readOnly = !novel;
  sqlInput.value = novel
    ? 'SELECT id, status FROM orders WHERE status = :status'
    : 'SELECT id, status, created_at\nFROM orders\nWHERE account_id = :account_id';
  approvalGroup.hidden = !novel;
  approvalInput.required = novel;
  approvalInput.value = '';
  modeDescription.textContent = novel
    ? `Novel SQL needs a one-use human code. Type ${challenge} before it can run.`
    : 'This named template is already approved by policy. Parameters remain bound values.';
  submitButton.textContent = novel ? 'Approve and run query' : 'Run named query';
  errorMessage.textContent = '';
}

modeButtons.forEach((button, index) => {
  button.addEventListener('click', () => setMode(button.dataset.mode as 'template' | 'novel'));
  button.addEventListener('keydown', (event) => {
    if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
    event.preventDefault();
    const next = event.key === 'ArrowRight' ? (index + 1) % modeButtons.length : (index - 1 + modeButtons.length) % modeButtons.length;
    modeButtons[next].click();
    modeButtons[next].focus();
  });
});

async function hash(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value.trim());
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

function loadReceipts(): DemoReceipt[] {
  try {
    const parsed = JSON.parse(localStorage.getItem(receiptsKey) ?? '[]') as unknown;
    return Array.isArray(parsed) ? parsed as DemoReceipt[] : [];
  } catch {
    return [];
  }
}

function saveReceipt(receipt: DemoReceipt): DemoReceipt[] {
  const history = [receipt, ...loadReceipts()].slice(0, 10);
  localStorage.setItem(receiptsKey, JSON.stringify(history));
  renderHistory(history);
  return history;
}

function renderHistory(history = loadReceipts()): void {
  historyList.replaceChildren();
  historyList.hidden = history.length === 0;
  clearButton.hidden = history.length === 0;
  required<HTMLElement>('#history-empty').hidden = history.length !== 0;
  for (const receipt of history) {
    const item = document.createElement('li');
    const outcome = document.createElement('span');
    outcome.className = `history-outcome ${receipt.outcome}`;
    outcome.textContent = receipt.outcome === 'allowed' ? '✓ Allowed' : '× Denied';
    const detail = document.createElement('span');
    detail.textContent = `${receipt.kind} · ${receipt.queryHash.slice(0, 10)}… · ${new Date(receipt.time).toLocaleString()}`;
    item.append(outcome, detail);
    historyList.append(item);
  }
}

function renderReceipt(receipt: DemoReceipt): void {
  emptyReceipt.hidden = true;
  receiptSheet.hidden = false;
  receiptSheet.classList.remove('receipt-enter');
  void receiptSheet.offsetWidth;
  receiptSheet.classList.add('receipt-enter');
  required<HTMLElement>('#receipt-status').textContent = receipt.outcome === 'allowed' ? '✓ Allowed and signed' : '× Denied and signed';
  required<HTMLElement>('#receipt-status').dataset.outcome = receipt.outcome;
  required<HTMLElement>('#receipt-id').textContent = receipt.id;
  required<HTMLElement>('#receipt-actor').textContent = receipt.actor;
  required<HTMLElement>('#receipt-kind').textContent = receipt.kind;
  required<HTMLElement>('#receipt-hash').textContent = `${receipt.queryHash.slice(0, 20)}…`;
  required<HTMLElement>('#receipt-approval').textContent = receipt.approval;
  required<HTMLElement>('#receipt-limits').textContent = `${receipt.rows}/${receipt.rowCap} rows · ${receipt.columnCap} columns max`;
  required<HTMLElement>('#receipt-reason').textContent = receipt.reason;
}

function sampleReceipt(): DemoReceipt {
  return {
    id: 'demo-20260827-001',
    time: '2026-08-27T09:14:00.000Z',
    actor: 'agent@northstar.example',
    kind: 'template',
    queryHash: '260ec10e8d9d368d89c61480b9e8f45ab08300a32b7ca917ea98bc9b970c2f43',
    outcome: 'allowed',
    approval: 'policy:open-orders',
    rows: 2,
    rowCap: 50,
    columnCap: 6,
    reason: 'Named sample query returned two matching orders.',
  };
}

function loadSampleDemo(): void {
  const history = loadReceipts();
  const receipt = history[0] ?? sampleReceipt();
  if (history.length === 0) saveReceipt(receipt);
  renderReceipt(receipt);
}

form.addEventListener('submit', async (event) => {
  event.preventDefault();
  errorMessage.textContent = '';
  submitButton.disabled = true;
  submitButton.textContent = 'Checking policy…';
  const sql = sqlInput.value;
  const classification = classifySql(sql);
  let outcome: DemoReceipt['outcome'] = 'allowed';
  let reason = 'Read-only query completed inside policy limits.';
  let approval = activeMode === 'template' ? 'policy:open-orders' : 'human-challenge';
  if (classification === 'empty') {
    outcome = 'denied';
    reason = 'No SQL was supplied.';
    approval = 'not-approved';
  } else if (classification === 'write') {
    outcome = 'denied';
    reason = 'Write or schema-changing SQL is never allowed.';
    approval = 'not-approved';
  } else if (activeMode === 'novel' && approvalInput.value.trim() !== challenge) {
    outcome = 'denied';
    reason = `Human code did not match. Type ${challenge} and try again.`;
    approval = 'not-approved';
  }
  const receipt: DemoReceipt = {
    id: crypto.randomUUID(),
    time: new Date().toISOString(),
    actor: actorInput.value.trim(),
    kind: activeMode,
    queryHash: await hash(sql),
    outcome,
    approval,
    rows: outcome === 'allowed' && accountInput.value.trim() ? 2 : 0,
    rowCap: activeMode === 'template' ? 50 : 100,
    columnCap: activeMode === 'template' ? 6 : 12,
    reason,
  };
  renderReceipt(receipt);
  saveReceipt(receipt);
  if (outcome === 'denied') errorMessage.textContent = `${reason} A denial receipt was still created.`;
  submitButton.disabled = false;
  submitButton.textContent = activeMode === 'novel' ? 'Approve and run query' : 'Run named query';
  receiptSheet.focus();
});

clearButton.addEventListener('click', () => {
  localStorage.removeItem(receiptsKey);
  renderHistory([]);
  receiptSheet.hidden = true;
  emptyReceipt.hidden = false;
  clearButton.textContent = isDemo ? 'Demo history cleared' : 'Local history cleared';
  setTimeout(() => { clearButton.textContent = isDemo ? 'Clear demo history' : 'Clear local history'; }, 1600);
});

const copyInstall = document.querySelector<HTMLButtonElement>('#copy-install');
copyInstall?.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText('git clone https://github.com/B-Divyesh/sf-db-access-receipts.git && cd sf-db-access-receipts && cargo install --path . --locked');
    copyInstall.textContent = 'Copied';
  } catch {
    copyInstall.textContent = 'Select command to copy';
  }
  setTimeout(() => { copyInstall.textContent = 'Copy setup command'; }, 1600);
});

function updateConnection(): void {
  const notice = required<HTMLElement>('#connection-status');
  notice.hidden = navigator.onLine;
  if (!navigator.onLine) notice.textContent = 'Offline — the guide and sample still work locally.';
}
window.addEventListener('online', updateConnection);
window.addEventListener('offline', updateConnection);
updateConnection();

const themeButton = required<HTMLButtonElement>('#theme-toggle');
const storedTheme = localStorage.getItem(themeKey);
if (storedTheme === 'light' || storedTheme === 'dark') document.documentElement.dataset.theme = storedTheme;
function updateThemeLabel(): void {
  const current = document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  themeButton.setAttribute('aria-label', `Use ${current === 'dark' ? 'light' : 'dark'} theme`);
}
themeButton.addEventListener('click', () => {
  const current = document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  document.documentElement.dataset.theme = current === 'dark' ? 'light' : 'dark';
  localStorage.setItem(themeKey, document.documentElement.dataset.theme);
  updateThemeLabel();
});
updateThemeLabel();

document.querySelector<HTMLButtonElement>('#reset-demo')?.addEventListener('click', () => {
  localStorage.removeItem(receiptsKey);
  localStorage.removeItem(themeKey);
  window.location.reload();
});

setMode('template');
if (isDemo) loadSampleDemo(); else renderHistory();

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/service-worker.js');
  });
}
