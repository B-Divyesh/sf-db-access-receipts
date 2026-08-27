import './styles.css';
import { cachedLicenseCanUnlock, classifySql, licenseCacheIsFresh } from './lib';

const PRODUCT_SLUG = 'db-access-receipts';
const BILLING_BASE = 'https://api.sociobot.in/api/v1';
const LICENSE_KEY = `sb_license:${PRODUCT_SLUG}`;
const VERDICT_KEY = `${LICENSE_KEY}:verdict`;
const RECEIPTS_KEY = 'db-receipts:demo-receipts';
const CHALLENGE = 'FERN-42';

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
    ? `Novel SQL needs a one-use human challenge. Type ${CHALLENGE} before it can run.`
    : 'This reviewed template is already approved by policy. Parameters remain bound values.';
  submitButton.textContent = novel ? 'Approve and run query' : 'Run allowlisted query';
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
    return JSON.parse(localStorage.getItem(RECEIPTS_KEY) ?? '[]') as DemoReceipt[];
  } catch {
    return [];
  }
}

function saveReceipt(receipt: DemoReceipt): void {
  const history = [receipt, ...loadReceipts()].slice(0, 10);
  localStorage.setItem(RECEIPTS_KEY, JSON.stringify(history));
  renderHistory(history);
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
  } else if (activeMode === 'novel' && approvalInput.value.trim() !== CHALLENGE) {
    outcome = 'denied';
    reason = `Human challenge did not match. Type ${CHALLENGE} and try again.`;
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
  submitButton.textContent = activeMode === 'novel' ? 'Approve and run query' : 'Run allowlisted query';
  receiptSheet.focus();
});

clearButton.addEventListener('click', () => {
  localStorage.removeItem(RECEIPTS_KEY);
  renderHistory([]);
  receiptSheet.hidden = true;
  emptyReceipt.hidden = false;
  clearButton.textContent = 'History cleared';
  setTimeout(() => { clearButton.textContent = 'Clear local history'; }, 1600);
});

required<HTMLButtonElement>('#copy-install').addEventListener('click', async (event) => {
  const button = event.currentTarget as HTMLButtonElement;
  try {
    await navigator.clipboard.writeText('cargo install db-access-receipts');
    button.textContent = 'Copied';
  } catch {
    button.textContent = 'Select command to copy';
  }
  setTimeout(() => { button.textContent = 'Copy install command'; }, 1600);
});

function updateConnection(): void {
  const notice = required<HTMLElement>('#connection-status');
  notice.hidden = navigator.onLine;
  if (!navigator.onLine) notice.textContent = 'Offline field mode — the guide and demo still work locally. License checks will resume when connected.';
}
window.addEventListener('online', updateConnection);
window.addEventListener('offline', updateConnection);
updateConnection();

const themeButton = required<HTMLButtonElement>('#theme-toggle');
const storedTheme = localStorage.getItem('db-receipts:theme');
if (storedTheme === 'light' || storedTheme === 'dark') document.documentElement.dataset.theme = storedTheme;
themeButton.addEventListener('click', () => {
  const current = document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  const next = current === 'dark' ? 'light' : 'dark';
  document.documentElement.dataset.theme = next;
  localStorage.setItem('db-receipts:theme', next);
  themeButton.setAttribute('aria-label', `Use ${next === 'dark' ? 'light' : 'dark'} theme`);
});

const licenseNotice = required<HTMLElement>('#license-notice');
const paidLocked = required<HTMLElement>('#paid-locked');
const paidUnlocked = required<HTMLElement>('#paid-unlocked');

function showUnlocked(unlocked: boolean, message = ''): void {
  paidLocked.hidden = unlocked;
  paidUnlocked.hidden = !unlocked;
  licenseNotice.textContent = message;
}

async function verifyLicense(token: string, force = false): Promise<void> {
  const cached = localStorage.getItem(VERDICT_KEY);
  if (!force && cachedLicenseCanUnlock(cached) && licenseCacheIsFresh(cached)) {
    showUnlocked(true, 'License verified on this device.');
    return;
  }
  try {
    const response = await fetch(`${BILLING_BASE}/products/${PRODUCT_SLUG}/verify?license=${encodeURIComponent(token)}`);
    if (!response.ok) throw new Error('Verification service unavailable');
    const verdict = await response.json() as { valid: boolean; reason: string };
    localStorage.setItem(VERDICT_KEY, JSON.stringify({ valid: verdict.valid, reason: verdict.reason, checkedAt: Date.now() }));
    showUnlocked(verdict.valid, verdict.valid ? 'License verified. The Team Field Kit is ready.' : 'License no longer active. You can purchase or restore another license.');
  } catch {
    const optimistic = cachedLicenseCanUnlock(cached);
    showUnlocked(optimistic, optimistic ? 'Offline — using the last verified license.' : 'Could not verify while offline. The free CLI and demo remain available.');
  }
}

const queryLicense = new URLSearchParams(location.search).get('license');
if (queryLicense) {
  localStorage.setItem(LICENSE_KEY, queryLicense);
  const clean = new URL(location.href);
  clean.searchParams.delete('license');
  history.replaceState({}, '', clean);
}
const storedLicense = queryLicense ?? localStorage.getItem(LICENSE_KEY);
if (storedLicense) {
  showUnlocked(cachedLicenseCanUnlock(localStorage.getItem(VERDICT_KEY)), 'Checking saved license…');
  void verifyLicense(storedLicense);
}

required<HTMLFormElement>('#restore-form').addEventListener('submit', (event) => {
  event.preventDefault();
  const input = required<HTMLInputElement>('#license-token');
  const token = input.value.trim();
  if (!token) return;
  localStorage.setItem(LICENSE_KEY, token);
  localStorage.removeItem(VERDICT_KEY);
  input.value = '';
  licenseNotice.textContent = 'Verifying license…';
  void verifyLicense(token, true);
});

required<HTMLButtonElement>('#download-kit').addEventListener('click', () => {
  const content = `DB Access Receipts — 30-day rollout field sheet\n\nWeek 1: Inventory readers and define named templates.\nWeek 2: Run in shadow mode and review denied receipts.\nWeek 3: Remove broad credentials; issue keychain-scoped access.\nWeek 4: Verify every pilot query has a signed receipt.\n\nDaily checks\n[ ] Zero write attempts passed\n[ ] Row and column caps matched query purpose\n[ ] Novel approvals name a human actor\n[ ] Receipt signatures verify offline\n`;
  const url = URL.createObjectURL(new Blob([content], { type: 'text/plain' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = 'db-access-receipts-team-field-kit.txt';
  link.click();
  URL.revokeObjectURL(url);
});

renderHistory();
setMode('template');

if ('serviceWorker' in navigator && import.meta.env.PROD) {
  window.addEventListener('load', () => {
    void navigator.serviceWorker.register('/service-worker.js');
  });
}
