import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const srcJsPath = path.join(__dirname, '../dist/ui/picker-overlay.js');
const outDir = path.join(__dirname, '../dist/overlay');

if (!fs.existsSync(outDir)) {
    fs.mkdirSync(outDir, { recursive: true });
}

console.log("Copying overlay bundle...");

try {
    // We copy the already compiled JS from 'npm run build'
    fs.copyFileSync(
        srcJsPath, 
        path.join(outDir, 'picker-overlay.js')
    );
    console.log("Overlay JS copied.");
    
    fs.copyFileSync(
        path.join(__dirname, '../src/ui/picker-overlay.css'), 
        path.join(outDir, 'picker-overlay.css')
    );
    console.log("Overlay CSS copied.");
} catch (e) {
    console.error("Failed to build overlay:", e.message);
}
