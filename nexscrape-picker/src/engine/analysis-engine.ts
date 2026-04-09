import { 
  ElementInfo, 
  GeneratedSelector, 
  SelectorType, 
  ElementAnalysisResult, 
  PickResult,
  FieldType,
  ExtractionStrategy
} from "../contracts.js";

import { SelectorGenerator } from "./selector-generator.js";
import { ElementAnalyzer } from "./element-analyzer.js";
import { Page } from "playwright";

export class AnalysisEngine {
  private selectorGen = new SelectorGenerator();
  private elementAnalyzer = new ElementAnalyzer();

  /**
   * Main entry point when an element is clicked.
   */
  async analyzeElement(info: ElementInfo, page: Page): Promise<PickResult> {
    
    console.log(`\nAnalyzing element: <${info.tag} id="${info.id || ''}" class="${info.classes.join(' ')}">`);
    
    // 1. Generate best possible selectors
    const selectors = this.selectorGen.generate(info);
    
    // 2. We can theoretically run match counts against the active page here via playwright
    for (const sel of selectors) {
        if (sel.type === "css" || sel.type === "attribute") {
            try {
                const count = await page.locator(sel.value).count();
                sel.matchCount = count;
                // If it matches exactly 1, heavily boost uniqueness score logic could go here
            } catch (e) {
                // Ignore parse errors from Playwright
                sel.matchCount = -1; 
            }
        }
    }
    
    // Sort again taking matchCount into account (prefer 1 over many, mostly)
    this.selectorGen.refineSorting(selectors);

    // 3. Analyze semantics (name, type, strategy)
    const analysis = this.elementAnalyzer.analyze(info, selectors[0]?.matchCount || 1);

    // 4. Assemble the result
    return {
      element: info,
      selectors,
      analysis,
      field: {
        id: "field_" + Date.now().toString(36),
        name: analysis.fieldName,
        type: analysis.fieldType,
        repeating: analysis.isRepeating,
        scope: analysis.suggestedScope,
        extraction: analysis.extraction,
        extractAttribute: analysis.extractAttribute,
        transform: analysis.transform,
        selector: {
          primary: selectors[0],
          fallbacks: selectors.slice(1)
        },
        preview: analysis.extraction === "attribute" && analysis.extractAttribute 
          ? (info.attributes[analysis.extractAttribute] || info.textContent).slice(0, 100).trim()
          : info.textContent.slice(0, 100).trim(),
        confidence: analysis.confidence,
        warnings: analysis.warnings
      }
    };
  }
}
