/**
 * NexPicker — Shared contracts and type definitions.
 *
 * These types define the interface between the picker overlay (browser),
 * the analysis engine (Node.js), and the NexScrape core library.
 * They are the single source of truth for all picker data structures.
 *
 * @module contracts
 */

// ─── Element Information ─────────────────────────────────────────────

/** Raw element data extracted from the DOM by the overlay. */
export interface ElementInfo {
  /** HTML tag name (lowercase). */
  tag: string;
  /** Element ID, if present. */
  id: string | null;
  /** CSS class list. */
  classes: string[];
  /** All HTML attributes. */
  attributes: Record<string, string>;
  /** Visible text content (trimmed, first 500 chars). */
  textContent: string;
  /** Inner HTML (first 2000 chars). */
  innerHTML: string;
  /** Outer HTML (first 3000 chars). */
  outerHTML: string;
  /** ARIA role, if set. */
  role: string | null;
  /** aria-label, if set. */
  ariaLabel: string | null;
  /** Bounding rect position. */
  rect: { x: number; y: number; width: number; height: number };
  /** Full DOM path from root — array of tag info objects. */
  domPath: DomPathSegment[];
  /** Parent element info (one level up). */
  parent: ParentInfo | null;
  /** Number of sibling elements with same tag. */
  siblingCount: number;
  /** Index among siblings (0-based). */
  siblingIndex: number;
  /** Number of direct children. */
  childCount: number;
  /** Computed styles that affect visibility. */
  computedStyle: {
    display: string;
    visibility: string;
    opacity: string;
    fontSize: string;
    fontWeight: string;
    color: string;
  };
}

/** A single segment in the DOM path from root to element. */
export interface DomPathSegment {
  tag: string;
  id: string | null;
  classes: string[];
  index: number;
  siblingCount: number;
}

/** Summary of parent element. */
export interface ParentInfo {
  tag: string;
  id: string | null;
  classes: string[];
  role: string | null;
  childCount: number;
}

// ─── Selector Types ──────────────────────────────────────────────────

/** Types of selectors the engine can generate. */
export type SelectorType = "css" | "xpath" | "text" | "role" | "attribute";

/** A generated selector with quality scores. */
export interface GeneratedSelector {
  /** The actual selector string. */
  value: string;
  /** Selector strategy type. */
  type: SelectorType;
  /** How resilient this selector is to DOM changes (0–100). */
  stability: number;
  /** How many elements it matches on the page (1 = perfectly unique). */
  matchCount: number;
  /** How human-readable it is (0–100). */
  readability: number;
  /** Is this a fallback rather than the primary suggestion? */
  isFallback: boolean;
  /** Optional warning about fragility. */
  warning?: string;
}

// ─── Field & Schema Types ────────────────────────────────────────────

/** Data types a field can hold. */
export type FieldType =
  | "string"
  | "number"
  | "url"
  | "image"
  | "date"
  | "boolean"
  | "html"
  | "list";

/** Extraction strategy for a field. */
export type ExtractionStrategy =
  | "text"
  | "attribute"
  | "innerHTML"
  | "outerHTML"
  | "count";

/** A single field definition in a NexScrape schema. */
export interface NexField {
  /** Unique field identifier. */
  id: string;
  /** Human-readable field name (snake_case). */
  name: string;
  /** Data type. */
  type: FieldType;
  /** Is this field inside a repeating container? */
  repeating: boolean;
  /** Parent container selector (scope). */
  scope: string | null;
  /** Extraction strategy. */
  extraction: ExtractionStrategy;
  /** HTML attribute to extract (for 'attribute' strategy). */
  extractAttribute?: string;
  /** Optional data transform (e.g. 'parseFloat', 'trim', 'absoluteUrl'). */
  transform?: string;
  /** Primary selector and fallback chain. */
  selector: {
    primary: GeneratedSelector;
    fallbacks: GeneratedSelector[];
  };
  /** Preview value from the live page. */
  preview: string;
  /** Overall confidence score (0–1). */
  confidence: number;
  /** Active warnings. */
  warnings: string[];
}

/** Pagination definition. */
export interface NexPagination {
  nextPage: {
    selector: string;
    type: SelectorType;
    attribute: string;
    stability: number;
  } | null;
}

/** Container (repeating item wrapper). */
export interface NexContainer {
  selector: string;
  type: SelectorType;
  count: number;
  stability: number;
}

/** Complete NexScrape extraction schema. */
export interface NexSchema {
  /** Schema version. */
  $schema: string;
  version: string;
  /** Source page URL. */
  page: string;
  /** When the schema was captured. */
  capturedAt: string;
  /** Project name (from nexscrape config). */
  project?: string;
  /** All extracted fields. */
  fields: NexField[];
  /** Pagination configuration. */
  pagination: NexPagination;
  /** Repeating item container. */
  container: NexContainer | null;
  /** Metadata. */
  metadata: {
    browserVersion: string;
    pickerVersion: string;
    pageTitle: string;
  };
}

// ─── Analysis Results ────────────────────────────────────────────────

/** Full analysis result returned after the user clicks an element. */
export interface ElementAnalysisResult {
  /** Suggested field name. */
  fieldName: string;
  /** Detected data type. */
  fieldType: FieldType;
  /** Recommended extraction strategy. */
  extraction: ExtractionStrategy;
  /** Attribute to extract, if relevant. */
  extractAttribute?: string;
  /** Transform function name. */
  transform?: string;
  /** Is it a repeating element? */
  isRepeating: boolean;
  /** Is it nested inside a card/container? */
  isNested: boolean;
  /** Suggested parent scope selector. */
  suggestedScope: string | null;
  /** Overall confidence (0–1). */
  confidence: number;
  /** Semantic hints derived from context. */
  semanticHints: string[];
  /** Active warnings. */
  warnings: string[];
}

// ─── Overlay ↔ Backend Messages ──────────────────────────────────────

/** Messages sent from the browser overlay to the Node.js backend. */
export type OverlayMessage =
  | { type: "element:hover"; data: { selector: string; preview: string } }
  | { type: "element:click"; data: ElementInfo }
  | { type: "element:deselect" }
  | { type: "picker:toggle"; data: { enabled: boolean } }
  | { type: "picker:exit" }
  | { type: "page:ready"; data: { url: string; title: string } }
  | { type: "scroll:position"; data: { x: number; y: number } };

/** Messages sent from the Node.js backend to the browser overlay. */
export type BackendMessage =
  | { type: "analysis:result"; data: PickResult }
  | { type: "selector:test"; data: { selector: string; matchCount: number } }
  | { type: "picker:enable" }
  | { type: "picker:disable" }
  | { type: "schema:saved"; data: { path: string } }
  | { type: "error"; data: { message: string } };

/** Complete pick result sent back after analysis. */
export interface PickResult {
  element: ElementInfo;
  selectors: GeneratedSelector[];
  analysis: ElementAnalysisResult;
  field: NexField;
}

// ─── Configuration ───────────────────────────────────────────────────

/** NexPicker launch configuration. */
export interface PickerConfig {
  /** URL to open. */
  url: string;
  /** Browser to use. */
  browser: "chromium" | "firefox" | "webkit";
  /** Viewport dimensions. */
  viewport: { width: number; height: number };
  /** Navigation timeout in ms. */
  timeout: number;
  /** Wait strategy after navigation. */
  waitStrategy: "load" | "domcontentloaded" | "networkidle" | "commit";
  /** Auto-save selections to schema. */
  autoSave: boolean;
  /** Output directory for schemas. */
  outputDir: string;
  /** Existing schema to append to. */
  existingSchema?: string;
  /** Check robots.txt compliance. */
  checkRobots: boolean;
  /** Custom User-Agent. */
  userAgent: string;
}

/** Default picker configuration. */
export const DEFAULT_PICKER_CONFIG: PickerConfig = {
  url: "",
  browser: "chromium",
  viewport: { width: 1366, height: 768 },
  timeout: 30000,
  waitStrategy: "networkidle",
  autoSave: true,
  outputDir: ".nexscrape",
  checkRobots: true,
  userAgent: "NexScrape-Picker/0.1.0 (Development Tool)",
};
