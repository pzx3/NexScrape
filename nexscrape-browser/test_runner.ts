import { BrowserRuntime } from "./src/index.js";
import { Response } from "playwright";

async function runDemo() {
  console.log("🚀 Starting NexScrape Browser Runtime Session...");

  // 1. تعريف المتصفح بحالة Headful لمشاهدة التفاعل الحي
  const browser = new BrowserRuntime({ 
      headless: false, 
      blockMedia: true // نعطل الصور لتسريع النتائج
  });

  try {
    const page = await browser.launch();

    // 2. مراقبة الشبكة في وضع الصيد للحصول على معلومات دقيقة بالخلفية
    page.watch.setupInterceptor({
      urlPattern: /api/, 
      onResponse: async (res: Response) => {
          if(res.status() === 200 && res.request().method() === "GET") {
              console.log(`[NETWORK SPY] Intercepted Data from: ${res.url()}`);
          }
      }
    });

    // 3. الدخول للموقع بانتظار `domcontentloaded`
    console.log("🌐 Navigating to complex target...");
    await page.visit("https://books.toscrape.com/", "domcontentloaded");

    // 4. السكرول للأسفل لإثبات التفاعل
    console.log("⏬ Scrolling to bottom...");
    await page.interact.scrollBottom();

    // 5. الضغط على زر أو رابط بطريقة تفاعلية ذكية
    console.log("🖱️ Clicking generic link...");
    await page.interact.click("a[href='catalogue/category/books/travel_2/index.html']");
    
    // 6. الانتظار الذكي لسكون الشبكة بعد النقر (لا نستخدم Sleep)
    console.log("⏳ Waiting for network to settle...");
    await page.wait.networkIdle(500);

    // 7. التقاط بيانات الـ DOM لتقديمها للمحرك الأساسي Core Engine
    console.log("📸 Taking DOM Snapshot for fast Core Extraction...");
    const htmlSnapshot = await page.getDomSnapshot();
    
    console.log(`✅ Success! Snapshot Size: ${htmlSnapshot.length} bytes`);

  } catch (err) {
    console.error("❌ Runtime Error:", err);
  } finally {
    await browser.close();
    console.log("🛑 Session Closed.");
  }
}

runDemo();
