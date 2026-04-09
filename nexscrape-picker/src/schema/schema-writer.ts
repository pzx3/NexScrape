import * as fs from "fs";
import * as path from "path";
import { NexSchema, NexField, PickerConfig } from "../contracts.js";

export class SchemaWriter {
  private config: PickerConfig;

  constructor(config: PickerConfig) {
    this.config = config;
  }

  writeField(field: NexField) {
    const schemaPath = this.getSchemaPath();
    let schema = this.loadSchema();

    // Check if field already exists, update or append
    const existingIndex = schema.fields.findIndex(f => f.name === field.name);
    if (existingIndex >= 0) {
      schema.fields[existingIndex] = field;
    } else {
      schema.fields.push(field);
    }
    
    // Auto-detect container logic (very simple heuristics for demo)
    if (field.repeating && field.scope && !schema.container) {
       schema.container = {
           selector: field.scope,
           type: "css",
           count: -1, // placeholder
           stability: field.selector.primary.stability
       };
    }

    schema.capturedAt = new Date().toISOString();
    
    fs.mkdirSync(path.dirname(schemaPath), { recursive: true });
    fs.writeFileSync(schemaPath, JSON.stringify(schema, null, 2), "utf8");
    console.log(`\n💾 Saved field '${field.name}' to ${schemaPath}`);
  }

  private loadSchema(): NexSchema {
    const schemaPath = this.getSchemaPath();
    if (fs.existsSync(schemaPath)) {
      try {
        const content = fs.readFileSync(schemaPath, "utf8");
        return JSON.parse(content) as NexSchema;
      } catch (e) {
        console.warn("Could not parse existing schema, creating new.");
      }
    }
    return this.createEmptySchema();
  }

  private getSchemaPath(): string {
    if (this.config.existingSchema) {
      return path.resolve(process.cwd(), this.config.existingSchema);
    }
    
    // Default naming convention: domain.schema.json inside outputDir
    try {
        const hostname = new URL(this.config.url).hostname.replace(/^www\./, "");
        return path.resolve(process.cwd(), this.config.outputDir, `${hostname}.schema.json`);
    } catch {
        return path.resolve(process.cwd(), this.config.outputDir, `default.schema.json`);
    }
  }

  private createEmptySchema(): NexSchema {
    return {
      $schema: "https://nexscrape.dev/schema/v1",
      version: "1.0",
      page: this.config.url,
      capturedAt: new Date().toISOString(),
      fields: [],
      pagination: { nextPage: null },
      container: null,
      metadata: {
        browserVersion: "Chromium (Playwright)",
        pickerVersion: "0.1.0",
        pageTitle: ""
      }
    };
  }
}
