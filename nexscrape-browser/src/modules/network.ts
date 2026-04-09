import { Page, Response } from "playwright";

export interface NetworkConfig {
  urlPattern: string | RegExp;
  onResponse: (response: Response) => Promise<void> | void;
}

/**
 * Network Module
 * Listens to underlying XHR/Fetch requests to capture JSON seamlessly.
 */
export class NetworkModule {
  private page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  /**
   * Monitor network for specific requests and capture their JSON payloads.
   * This is extremely powerful for bypassing DOM parsing entirely in SPAs.
   */
  async setupInterceptor(config: NetworkConfig): Promise<void> {
    this.page.on("response", async (response) => {
      const url = response.url();
      const isMatch = typeof config.urlPattern === "string" 
        ? url.includes(config.urlPattern)
        : config.urlPattern.test(url);

      if (isMatch) {
         await config.onResponse(response);
      }
    });
  }
}
