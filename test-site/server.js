import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const port = 3000;

app.use(express.static('public'));

// API Endpoint لمطالبة البيانات بشكل ديناميكي (AJAX)
app.get('/api/products', (req, res) => {
    // محاكاة تأخير الشبكة 1.5 ثانية (Lazy Loading)
    setTimeout(() => {
        res.json({
            status: "success",
            data: [
                { id: 1, name: "حذاء رياضي متطور", price: 299, inStock: true },
                { id: 2, name: "ساعة ذكية بشاشة OLED", price: 899, inStock: false },
                { id: 3, name: "نظارة شمسية صيفية", price: 150, inStock: true }
            ]
        });
    }, 1500);
});

app.listen(port, () => {
  console.log(`[TEST SITE] Running at http://localhost:${port}`);
});
