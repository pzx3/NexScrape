import { BrowserRuntime } from "../nexscrape-browser/src/index.js";

/**
 * NexScrape Library Example - Manual Scraping
 * This script demonstrates using the Node.js library to extract data 
 * from the local test site.
 */
async function run() {
  console.log("🚀 Starting NexScrape Library Example...");

  // 1. Initialize the runtime (Playwright based)
  const browser = new BrowserRuntime({ 
    headless: true,
    blockMedia: true 
  });

  try {
    const page = await browser.launch();

    // 2. Navigate to the test site
    console.log("🌐 Navigating to test site...");
    await page.navigate("http://localhost:3000");

    // 3. Extract data using page.evaluate (Universal & Powerful)
    const data = await page.evaluate(() => {
      const title = document.querySelector('h1')?.textContent?.trim();
      const products = Array.from(document.querySelectorAll('.product-item')).map(el => ({
        name: el.querySelector('.item-title')?.textContent?.trim(),
        price: el.querySelector('.price')?.textContent?.trim(),
        link: el.querySelector('a')?.getAttribute('href')
      }));

      return { title, products };
    });

    console.log("\n✅ Extraction Success!");
    console.log("-------------------");
    console.log(`Page Title: ${data.title}`);
    console.log(`Found ${data.products.length} products:`);
    data.products.forEach((p, i) => {
        console.log(`  ${i+1}. ${p.name} - ${p.price}`);
    });

  } catch (error) {
    console.error("❌ Scraping failed:", error);
  } finally {
    // 4. Always close the browser
    await browser.close();
    console.log("\n🔚 Browser closed.");
  }
}

run();
