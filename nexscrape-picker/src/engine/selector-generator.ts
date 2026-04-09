import { ElementInfo, GeneratedSelector } from "../contracts.js";

export class SelectorGenerator {
  generate(element: ElementInfo): GeneratedSelector[] {
    const candidates: GeneratedSelector[] = [];

    // 1. ID based
    if (element.id && !this.isGeneratedString(element.id)) {
      candidates.push({
        value: `#${element.id}`,
        type: "css",
        stability: 95,
        matchCount: 0,
        readability: 95,
        isFallback: false
      });
    }

    // 2. Data attributes (e.g. data-testid, data-cy)
    const testAttr = this.findTestAttribute(element);
    if (testAttr) {
      candidates.push({
        value: `[${testAttr.name}="${testAttr.value}"]`,
        type: "attribute",
        stability: 95,
        matchCount: 0,
        readability: 90,
        isFallback: false
      });
    }

    // 3. Specific accessibility roles
    if (element.role && element.ariaLabel) {
      candidates.push({
        value: `[role="${element.role}"][aria-label="${element.ariaLabel}"]`,
        type: "role",
        stability: 85,
        matchCount: 0,
        readability: 85,
        isFallback: false
      });
    }

    // 4. Semantic CSS Class
    const cssClassSel = this.buildSemanticCss(element);
    if (cssClassSel) {
      candidates.push({
        value: cssClassSel,
        type: "css",
        stability: this.analyzeClassesStability(element.classes),
        matchCount: 0,
        readability: 80,
        isFallback: false
      });
    }
    
    // 5. Parent scope + semantic child
    const parentCss = element.parent ? this.buildSemanticCss({ ...element.parent, attributes: {} } as any) : null;
    if (parentCss && cssClassSel) {
        candidates.push({
            value: `${parentCss} > ${cssClassSel}`,
            type: "css",
            stability: Math.min(this.analyzeClassesStability(element.classes), this.analyzeClassesStability(element.parent!.classes)),
            matchCount: 0,
            readability: 70,
            isFallback: true
        });
    }

    // 6. Text based locator (Playwright style)
    if (element.textContent && element.textContent.length > 2 && element.textContent.length < 50) {
      candidates.push({
        value: `text="${element.textContent.trim()}"`,
        type: "text",
        stability: 50, // Sensitive to content changes
        matchCount: 0,
        readability: 90,
        isFallback: true,
        warning: "Text locators break if content translation or text changes occur."
      });
    }

    // 7. Structural XPath (Last resort)
    candidates.push({
      value: this.buildStructuralXPath(element),
      type: "xpath",
      stability: 30, // Breaks easily on layout changes
      matchCount: 0,
      readability: 30,
      isFallback: true,
      warning: "Highly fragile structural XPath."
    });

    // Ensure we don't have duplicate string values
    const unique = [];
    const seen = new Set<string>();
    for (const c of candidates) {
        if (!seen.has(c.value)) {
            seen.add(c.value);
            unique.push(c);
        }
    }

    // Initial Sort
    return unique.sort(this.scoreComparator);
  }

  refineSorting(selectors: GeneratedSelector[]) {
     // Rule: if multiple matches, its not a good unique identifier unless repeating scope
     // (We handle repetitions in ElementAnalyzer, so here we just sort generally)
     selectors.sort(this.scoreComparator);
  }

  private scoreComparator(a: GeneratedSelector, b: GeneratedSelector) {
     const scoreA = (a.stability * 0.6) + (a.readability * 0.4);
     const scoreB = (b.stability * 0.6) + (b.readability * 0.4);
     return scoreB - scoreA;
  }

  private findTestAttribute(element: ElementInfo) {
    const testKeys = ['data-testid', 'data-test-id', 'data-cy', 'data-qa'];
    for (const k of testKeys) {
      if (element.attributes[k]) {
        return { name: k, value: element.attributes[k] };
      }
    }
    return null;
  }

  private analyzeClassesStability(classes: string[]): number {
    if (!classes || classes.length === 0) return 40;
    
    // Tailwind/CSS Modules/Styled-components generated classes detection
    const generatedPattern = /^[a-f0-9]{5,}$|-[a-z0-9]{4,6}$|^css-/i;
    const utilityPattern = /^mt-|mb-|p-|flex|block|text-|bg-|grid|w-/;
    
    const semantic = classes.filter(c => !generatedPattern.test(c) && !utilityPattern.test(c));
    if (semantic.length === 0) return 30; // Only utility or generated
    return 80;
  }

  private buildSemanticCss(element: ElementInfo): string | null {
    if (!element.classes || element.classes.length === 0) return element.tag;

    const generatedPattern = /^[a-f0-9]{5,}$|-[a-z0-9]{4,6}$|^css-/i;
    const utilityPattern = /^mt-|mb-|p-|flex|block|text-|bg-|grid|w-|h-|rounded|shadow|border/i;

    const stable = element.classes.filter(c => !generatedPattern.test(c) && !utilityPattern.test(c));
    
    if (stable.length > 0) {
      // Pick the first, most descriptive stable class
      return `${element.tag}.${stable[0]}`;
    }
    
    return element.tag; // fallback to tag name
  }

  private isGeneratedString(str: string): boolean {
    return /^[a-f0-9]{8,}|[0-9]{5,}/i.test(str);
  }

  private buildStructuralXPath(element: ElementInfo): string {
    const segments = [];
    for (const seg of element.domPath) {
        if (seg.id && !this.isGeneratedString(seg.id)) {
            segments.unshift(`//${seg.tag}[@id='${seg.id}']`);
            break; // Stop climbing if we hit a solid ID
        } else {
            segments.unshift(`${seg.tag}[${seg.index + 1}]`);
        }
    }
    if (!segments[0].startsWith("//")) {
        segments[0] = "//" + segments[0];
    }
    return segments.join("/");
  }
}
