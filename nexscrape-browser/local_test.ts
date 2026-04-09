import { BrowserRuntime } from "./src/index.js";
import { Response } from "playwright";

async function testLocalSpa() {
  console.log("🚀 Starting Local Test against SPA...");

  const browser = new BrowserRuntime({ 
      headless: false,
      blockMedia: false
  });

  try {
    const page = await browser.launch();

    // نضع فخ لمراقبة الـ API خلف الأضواء (Secret Fetching Intercept)
    page.watch.setupInterceptor({
      urlPattern: /api\/products/,
      onResponse: async (res: Response) => {
          if (res.status() === 200) {
              const json = await res.json();
              console.log("🕵️‍♂️ [NETWORK SPY] Caught JSON payload directly!", JSON.stringify(json.data[0]));
          }
      }
    });

    console.log("🌐 Navigating to localhost:3000 ...");
    await page.visit("http://localhost:3000/", "domcontentloaded");

    // نتحقق من وجود زر تحميل المزيد
    console.log("🖱️ Clicking 'Load More' button...");
    await page.interact.click("#load-more-btn");
    
    // الانتظار المعماري الذكي (بدلاً من setTimeout)
    console.log("⏳ Waiting for API response and DOM injection (Network Idle)...");
    await page.wait.networkIdle(500); // ينتظر حتى يتوقف الرصيد الشبكي لـ 500ms
    
    console.log("🔍 Looking for injected DOM Elements...");
    
    // استخراج الكود باستخدام JS
    const productsCount = await page.runScript(() => {
        return document.querySelectorAll(".product-card").length;
    });

    console.log(`✅ Success! Found ${productsCount} product cards injected dynamically into the DOM!`);
    
    // أخذ لقطة لمحرك Rust لاستخراج دقيق
    const html = await page.getDomSnapshot();
    console.log(`📦 Final Snapshot Size for Rust Parser: ${html.length} bytes`);

  } catch (err) {
    console.error("❌ Test Failed:", err);
  } finally {
    await browser.close();
    console.log("🛑 Test Finished.");
  }
}

testLocalSpa();
