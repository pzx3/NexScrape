import { Page } from "playwright";

/**
 * Smart Wait Strategies for heavily dynamic pages.
 */
export class WaitModule {
  private page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Waits for the network to be completely idle for a given timeframe.
   * Super useful for SPA rendering delays.
   */
  async networkIdle(timeoutMs: number = 500): Promise<void> {
    await this.page.waitForLoadState("networkidle", { timeout: timeoutMs + 30000 }).catch(() => {});
    // manual sleep for the specific chunk if the internal wasn't enough
    await this.page.waitForTimeout(timeoutMs);
  }

  /**
   * Ensures the given selector is visible globally on the page.
   */
  async untilVisible(selector: string, timeout: number = 10000): Promise<void> {
    await this.page.waitForSelector(selector, { state: "visible", timeout });
  }

  /**
   * Advanced wait for React, Vue, Next.js hydration.
   * It probes for internal framework object existence.
   */
  async untilHydrated(timeout: number = 15000): Promise<void> {
    await this.page.waitForFunction(() => {
      // Check for Vue
      const vueApp = document.querySelector('[data-v-app]');
      if (vueApp && (vueApp as any).__vue_app__) return true;
      
      // Check for React / Next.js
      const nextData = document.getElementById('__NEXT_DATA__');
      if (nextData) return true;
      
      const reactRoot = document.querySelector('[data-reactroot]');
      if (reactRoot && (reactRoot as any)._reactInternals) return true;
      
      // Fallback simple condition
      return document.body.innerHTML.length > 500 && !document.querySelector('.loading, .spinner');
    }, null, { timeout });
  }

  /**
   * Waits until the text content of a selector changes from its old value.
   */
  async textChanges(selector: string, oldText: string, timeout: number = 10000): Promise<void> {
    await this.page.waitForFunction((args) => {
      const el = document.querySelector(args.sel);
      return el && el.textContent?.trim() !== args.old;
    }, { sel: selector, old: oldText }, { timeout });
  }
}
