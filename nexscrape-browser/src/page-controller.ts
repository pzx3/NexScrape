import { Page as PlaywrightPage } from "playwright";
import { WaitModule } from "./modules/wait.js";
import { InteractModule } from "./modules/interact.js";
import { NetworkModule } from "./modules/network.js";

/**
 * NexScrape Page Controller
 * The main interface for developers to interact with a dynamic web page.
 */
export class PageController {
  private page: PlaywrightPage;
  
  public wait: WaitModule;
  public interact: InteractModule;
  public watch: NetworkModule;

  constructor(page: PlaywrightPage) {
    this.page = page;
    
    // Initialize DSL modules
    this.wait = new WaitModule(this.page);
    this.interact = new InteractModule(this.page);
    this.watch = new NetworkModule(this.page);
  }

  /**
   * Navigate to a target URL
   */
  async visit(url: string, waitUntil: "load" | "domcontentloaded" | "networkidle" = "domcontentloaded"): Promise<void> {
    console.log(`[NexScrape Browser] Visiting: ${url}`);
    await this.page.goto(url, { waitUntil });
  }

  /**
   * Execute custom JavaScript within the page context and return the result.
   */
  async runScript<T>(script: string | ((...args: any[]) => T), ...args: any[]): Promise<T> {
    return this.page.evaluate(script, ...args);
  }

  /**
   * Captures the current DOM and returns it as a string.
   * This snapshot can be immediately piped to the fast Rust Core for extraction.
   */
  async getDomSnapshot(): Promise<string> {
    return this.page.content();
  }
}
