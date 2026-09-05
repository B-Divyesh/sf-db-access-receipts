import './styles.css';

const themeButton = document.querySelector<HTMLButtonElement>('#theme-toggle');
const themeKey = 'db-receipts:theme';
const storedTheme = localStorage.getItem(themeKey);
if (storedTheme === 'light' || storedTheme === 'dark') document.documentElement.dataset.theme = storedTheme;

function updateThemeLabel(): void {
  if (!themeButton) return;
  const current = document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  themeButton.setAttribute('aria-label', `Use ${current === 'dark' ? 'light' : 'dark'} theme`);
}

themeButton?.addEventListener('click', () => {
  const current = document.documentElement.dataset.theme ?? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
  document.documentElement.dataset.theme = current === 'dark' ? 'light' : 'dark';
  localStorage.setItem(themeKey, document.documentElement.dataset.theme);
  updateThemeLabel();
});

updateThemeLabel();
