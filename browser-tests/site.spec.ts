import { expect, test } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('@claim:demo-sandbox loads a populated isolated sample and can reset', async ({ page }) => {
  await page.goto('/demo/');
  await expect(page).toHaveTitle('Demo — DB Access Receipts');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review a sample SQLite access receipt');
  await expect(page.getByText('Demo — sample data, nothing is saved to your real data.')).toBeVisible();
  await expect(page.locator('#receipt-status')).toHaveText('✓ Allowed and signed');
  await expect(page.locator('#receipt-actor')).toHaveText('agent@northstar.example');
  await expect(page.locator('#receipt-limits')).toHaveText('2/50 rows · 6 columns max');
  expect(await page.evaluate(() => Object.keys(localStorage))).toContain('demo:db-receipts:receipts');
  expect(await page.evaluate(() => localStorage.getItem('db-receipts:receipts'))).toBeNull();

  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('#receipt-status')).toHaveText('✓ Allowed and signed');
  await page.getByRole('link', { name: 'Start for real' }).click();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByText('Demo — sample data, nothing is saved to your real data.')).toHaveCount(0);
});

test('@claim:local-only-browser keeps the sample flow on this origin', async ({ page, baseURL }) => {
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await page.goto('/demo/');
  await page.getByRole('tab', { name: 'Novel SQL' }).click();
  await page.locator('#approval').fill('wrong');
  await page.getByRole('button', { name: 'Approve and run query' }).click();
  await expect(page.locator('#receipt-status')).toHaveText('× Denied and signed');
  const expectedOrigin = new URL(baseURL ?? 'http://127.0.0.1:4173').origin;
  expect(requests.length).toBeGreaterThan(0);
  expect(requests.every((url) => new URL(url).origin === expectedOrigin)).toBe(true);
});

test('@claim:offline-demo works offline after its first visit in a fresh browser context', async ({ browser, baseURL }) => {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await page.goto(`${baseURL}/demo/`);
    await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
    await page.reload();
    await page.waitForFunction(() => navigator.serviceWorker.controller !== null);
    await context.setOffline(true);
    await page.goto(`${baseURL}/demo/`, { waitUntil: 'domcontentloaded' });
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review a sample SQLite access receipt');
    await expect(page.locator('#receipt-status')).toHaveText('✓ Allowed and signed');
  } finally {
    await context.close();
  }
});

test('routes meet accessibility, metadata, keyboard, and touch-target baselines', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  const routes = [
    ['/', 'DB Access Receipts — Gate SQLite reads'],
    ['/demo/', 'Demo — DB Access Receipts'],
    ['/privacy/', 'Privacy — DB Access Receipts'],
    ['/terms/', 'Terms — DB Access Receipts'],
    ['/404.html', 'Page not found — DB Access Receipts'],
  ] as const;
  for (const [route, title] of routes) {
    await page.goto(route);
    await expect(page).toHaveTitle(title);
    await expect(page.locator('html')).toHaveAttribute('lang', 'en');
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveCount(1);
    const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
    const serious = results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''));
    expect(serious.map((violation) => violation.id), JSON.stringify(serious, null, 2)).toEqual([]);
  }
  await page.goto('/');
  await page.keyboard.press('Tab');
  await expect(page.locator('.skip-link')).toBeFocused();
  for (const element of await page.locator('.site-nav a, .site-nav button, .footer-links a').all()) {
    const box = await element.boundingBox();
    if (box) expect(box.height).toBeGreaterThanOrEqual(44);
  }
  await page.emulateMedia({ reducedMotion: 'reduce' });
  expect(await page.locator('html').evaluate((element) => getComputedStyle(element).scrollBehavior)).toBe('auto');
  await page.goto('/demo/');
  await page.getByRole('button', { name: 'Use dark theme' }).click();
  const darkResults = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  const darkSerious = darkResults.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? ''));
  expect(darkSerious.map((violation) => violation.id), JSON.stringify(darkSerious, null, 2)).toEqual([]);
});
