import { createRoot } from 'octane';
import '@fontsource/dm-sans/400.css';
import '@fontsource/dm-sans/500.css';
import '@fontsource/dm-sans/600.css';
import '@fontsource/dm-sans/700.css';
import '@fontsource/jetbrains-mono/400.css';
import '@fontsource/jetbrains-mono/500.css';
import { App } from './App.tsrx';
import './styles.css';

const container = document.getElementById('app');
if (!container) {
  throw new Error('Missing #app root');
}

const root = createRoot(container);
root.render(App, {});
