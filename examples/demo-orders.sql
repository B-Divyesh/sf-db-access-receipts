CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  account_id TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL
);

INSERT INTO orders (id, account_id, status, created_at) VALUES
  (101, 'acct_demo', 'open', '2026-08-24T09:14:00Z'),
  (102, 'acct_demo', 'review', '2026-08-25T13:42:00Z'),
  (103, 'acct_other', 'closed', '2026-08-26T16:03:00Z');
