import { createRoot } from 'react-dom/client';
import App from './App';

const rootElement = document.getElementById('utopia');

if (rootElement === null) {
    throw new Error('No root element found on page');
}

const root = createRoot(rootElement);
root.render(<App />);
