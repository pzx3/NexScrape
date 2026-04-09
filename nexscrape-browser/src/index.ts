export * from "./page-controller.js";
import { chromium, Browser } from "playwright";
import { PageController } from "./page-controller.js";

/**
 * Interface for module configuration
 */
export interface RuntimeConfig {
  headless: boolean;
  blockMedia: boolean;
  timeoutMs: number;
}

/**
 * Main Orchestrator to launch NexScrape JavaScript environment
 */
export class BrowserRuntime {
  private browser: Browser | null = null;
  private config: RuntimeConfig;

  constructor(config: Partial<RuntimeConfig> = {}) {
    this.config = {
      headless: config.headless ?? true,
      blockMedia: config.blockMedia ?? true,
      timeoutMs: config.timeoutMs ?? 30000
    };
  }

  async launch(): Promise<PageController> {
    this.browser = await chromium.launch({
      headless: this.config.headless,
      args: ['--disable-blink-features=AutomationControlled']
    });

    const context = await this.browser.newContext();

    if (this.config.blockMedia) {
      await context.route("**/*", (route) => {
        const type = route.request().resourceType();
        if (["image", "media", "font", "stylesheet"].includes(type)) {
          route.abort();
        } else {
          route.continue();
        }
      });
    }

    const page = await context.newPage();
    page.setDefaultTimeout(this.config.timeoutMs);

    return new PageController(page);
  }

  async close(): Promise<void> {
    if (this.browser) {
      await this.browser.close();
    }
  }
}
