import * as React from 'react';
import { createRoot } from 'react-dom/client';

const rootElement = document.getElementById('utopia');

if (rootElement === null) {
    throw new Error('No root element found on page');
}

const root = createRoot(rootElement);
root.render(<h1>Hello, TypeScript</h1>);
