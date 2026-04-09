import { ElementInfo, ElementAnalysisResult } from "../contracts.js";

export class ElementAnalyzer {
  analyze(element: ElementInfo, matchCount: number): ElementAnalysisResult {
    const fieldName = this.inferFieldName(element);
    const fieldType = this.inferType(element);
    const extraction = this.inferExtractionStrategy(element, fieldType);
    
    const isRepeating = matchCount > 1 || element.siblingCount > 1;
    const isNested = this.detectNestedCard(element);
    
    const warnings: string[] = [];
    if (element.classes.some(c => /css-[a-z0-9]+/i.test(c))) {
        warnings.push("Element uses generated CSS classes. Visual updates to the site might break selectors.");
    }
    if (matchCount > 1 && !isNested) {
        warnings.push(`Matched ${matchCount} items on the page. You may need to select a parent container first.`);
    }

    return {
      fieldName,
      fieldType,
      extraction,
      extractAttribute: this.inferExtractionAttribute(element, extraction),
      transform: this.inferTransform(fieldType),
      isRepeating,
      isNested,
      suggestedScope: isNested ? this.findParentContainer(element) : null,
      confidence: this.calculateConfidence(element, fieldName),
      semanticHints: this.extractSemanticHints(element),
      warnings
    };
  }

  private inferFieldName(el: ElementInfo): string {
    if (el.ariaLabel) return this.slugify(el.ariaLabel);

    const testId = el.attributes["data-testid"] || el.attributes["data-cy"] || el.attributes["data-qa"];
    if (testId) return this.slugify(testId);

    const match = el.classes.find(c => /title|price|name|img|image|description|author|date|link/i.test(c));
    if (match) return this.slugify(match);

    const tagToName: Record<string, string> = {
      h1: "main_title", h2: "subtitle", h3: "sub_heading",
      img: "image_url", a: "link_url", time: "date",
    };
    return tagToName[el.tag] || `field_${el.tag}`;
  }

  private inferType(el: ElementInfo): ElementAnalysisResult["fieldType"] {
    if (el.tag === "img") return "image";
    if (el.tag === "a") return "url";
    if (el.tag === "time") return "date";
    if (el.attributes["type"] === "checkbox") return "boolean";
    if (el.textContent && /^\$?\d+(\.\d{1,2})?$/.test(el.textContent.trim())) return "number";
    return "string";
  }

  private inferExtractionStrategy(el: ElementInfo, type: string): "text" | "attribute" | "innerHTML" {
    if (type === "image" || el.tag === "img") return "attribute";
    if (type === "url" || el.tag === "a") return "attribute";
    return "text";
  }

  private inferExtractionAttribute(el: ElementInfo, strategy: string): string | undefined {
    if (strategy !== "attribute") return undefined;
    if (el.tag === "img") return "src";
    if (el.tag === "a") return "href";
    return "value";
  }

  private inferTransform(type: string): string | undefined {
    if (type === "number") return "parseFloat";
    if (type === "url" || type === "image") return "absoluteUrl";
    return "trim";
  }

  private detectNestedCard(el: ElementInfo): boolean {
    const cardPatterns = /card|item|product|listing|row/i;
    let curr = el.parent;
    for(let i=0; i<3; i++) { // look up 3 levels
        if (!curr) break;
        if (curr.classes.some(c => cardPatterns.test(c))) return true;
        // In this implementation we don't have full parent path deeply accessible without passing it,
        // so we check just immediate parent info passed from overlay
    }
    return false;
  }

  private findParentContainer(el: ElementInfo): string | null {
    if (!el.parent) return null;
    if (el.parent.classes.length > 0) {
        return `${el.parent.tag}.${el.parent.classes[0]}`;
    }
    return el.parent.tag;
  }

  private slugify(str: string) {
    return str.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '');
  }

  private calculateConfidence(el: ElementInfo, name: string): number {
    let conf = 0.5;
    if (name !== `field_${el.tag}`) conf += 0.2;
    if (el.id) conf += 0.2;
    if (el.classes.length > 0) conf += 0.1;
    return Math.min(1.0, conf);
  }

  private extractSemanticHints(el: ElementInfo): string[] {
    const hints = [];
    if (/price|cost|amount/i.test(el.textContent) || el.textContent.includes("$")) hints.push("money");
    if (el.tag === "a") hints.push("navigation");
    return hints;
  }
}
