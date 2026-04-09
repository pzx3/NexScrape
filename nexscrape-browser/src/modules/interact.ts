import { Page } from "playwright";

/**
 * Handles all user-like interactions with the page.
 */
export class InteractModule {
  private page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Click an element, simulating human behavior.
   */
  async click(selector: string, delay: number = 50): Promise<void> {
    await this.page.click(selector, { delay });
  }

  /**
   * Type text into an input field smoothly.
   */
  async type(selector: string, text: string, delayBetweenStrokes: number = 30): Promise<void> {
    await this.page.type(selector, text, { delay: delayBetweenStrokes });
  }

  /**
   * Scroll to the very bottom of the page, triggering infinite scrolls.
   */
  async scrollBottom(): Promise<void> {
    await this.page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  }

  /**
   * Scroll to a specific element.
   */
  async scrollTo(selector: string): Promise<void> {
    await this.page.locator(selector).scrollIntoViewIfNeeded();
  }
}
