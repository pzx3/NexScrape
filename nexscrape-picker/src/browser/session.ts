import { chromium, Browser, BrowserContext, Page } from "playwright";
import { WebSocketServer, WebSocket } from "ws";
import * as http from "http";
import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";
import { PickerConfig, OverlayMessage, BackendMessage } from "../contracts.js";
import { AnalysisEngine } from "../engine/analysis-engine.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export interface BrowserSessionOptions {
  config: PickerConfig;
  onOverlayMessage: (msg: OverlayMessage) => void;
  onPageClose: () => void;
}

export class BrowserSession {
  private browser: Browser | null = null;
  private context: BrowserContext | null = null;
  private page: Page | null = null;
  private wss: WebSocketServer | null = null;
  private httpServer: http.Server | null = null;
  private activeWs: WebSocket | null = null;
  private wsPort: number = 0;
  
  private options: BrowserSessionOptions;

  constructor(options: BrowserSessionOptions) {
    this.options = options;
  }

  /** Starts the browser and the internal WebSocket server. */
  async launch(): Promise<Page> {
    // 1. Start WS server for overlay communication
    await this.startWebSocketServer();

    // 2. Launch browser
    this.browser = await chromium.launch({
      headless: false, // Must be headful for visual picking
      args: [
        "--disable-blink-features=AutomationControlled",
        "--window-size=" + this.options.config.viewport.width + "," + this.options.config.viewport.height,
        "--disable-infobars"
      ],
    });

    this.context = await this.browser.newContext({
      bypassCSP: true,
      viewport: this.options.config.viewport,
      userAgent: this.options.config.userAgent,
    });

    this.page = await this.context.newPage();

    this.page.on("close", () => {
      this.options.onPageClose();
    });

    // Handle persistent injection on navigation
    this.page.on("load", async () => {
        try {
            await this.injectOverlay();
        } catch (e) {
            console.error("Failed to re-inject overlay on navigation:", e);
        }
    });

    // PREVENTION: Ensure all links open in the same tab
    this.context.on("page", async (newPage) => {
        if (newPage !== this.page && this.page) {
            const url = newPage.url();
            if (url !== "about:blank") {
                console.log(`[Single-Tab Mode] Redirecting main page to: ${url}`);
                await this.page.goto(url).catch(() => {});
            }
            await newPage.close().catch(() => {});
        }
    });

    return this.page;
  }

  /** Navigates to the URL. (Injection now happens automatically via 'load' event) */
  async navigateAndInject(): Promise<void> {
    if (!this.page) throw new Error("Browser not launched");

    console.log(`\nNavigating to ${this.options.config.url}...`);
    
    await this.page.goto(this.options.config.url, {
      waitUntil: this.options.config.waitStrategy,
      timeout: this.options.config.timeout,
    });
  }

  /** Core injection logic for JS and CSS */
  private async injectOverlay(): Promise<void> {
    if (!this.page) return;

    console.log(`Injecting NexPicker overlay...`);

    // We pass the WS port to the overlay so it can connect back to Node
    await this.page.evaluate((port) => {
      (window as any).__NEX_WS_PORT = port;
    }, this.wsPort);

    // Inject CSS
    const cssPath = path.join(__dirname, "../../dist/overlay/picker-overlay.css");
    if (fs.existsSync(cssPath)) {
      await this.page.addStyleTag({ path: cssPath });
    }

    // Inject JS
    const jsPath = path.join(__dirname, "../../dist/overlay/picker-overlay.js");
    if (fs.existsSync(jsPath)) {
        await this.page.addScriptTag({ path: jsPath, type: "module" });
    }

    console.log(`Overlay injected successfully.`);
  }

  /** Sends a message to the browser overlay. */
  sendMessage(msg: BackendMessage): void {
    if (this.activeWs && this.activeWs.readyState === WebSocket.OPEN) {
      this.activeWs.send(JSON.stringify(msg));
    }
  }

  /** Closes the browser and servers. */
  async close(): Promise<void> {
    if (this.browser) await this.browser.close();
    if (this.wss) this.wss.close();
    if (this.httpServer) this.httpServer.close();
  }

  private startWebSocketServer(): Promise<void> {
    return new Promise((resolve) => {
      this.httpServer = http.createServer();
      this.wss = new WebSocketServer({ server: this.httpServer });

      this.wss.on("connection", (ws) => {
        this.activeWs = ws;
        
        ws.on("message", (message) => {
          try {
            const data = JSON.parse(message.toString()) as OverlayMessage;
            this.options.onOverlayMessage(data);
          } catch (e) {
            console.error("Failed to parse overlay message:", e);
          }
        });

        ws.on("close", () => {
          if (this.activeWs === ws) this.activeWs = null;
        });
      });

      // Listen on random free port
      this.httpServer.listen(0, "127.0.0.1", () => {
        const addr = this.httpServer?.address();
        if (addr && typeof addr === "object") {
          this.wsPort = addr.port;
          resolve();
        }
      });
    });
  }
}
