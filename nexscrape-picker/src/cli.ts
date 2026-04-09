#!/usr/bin/env node

/**
 * NexPicker CLI Entry Point
 */

import { Command } from "commander";
import { BrowserSession } from "./browser/session.js";
import { AnalysisEngine } from "./engine/analysis-engine.js";
import { SchemaWriter } from "./schema/schema-writer.js";
import { DEFAULT_PICKER_CONFIG, OverlayMessage } from "./contracts.js";
import chalk from "chalk";
import ora from "ora";
import * as fs from "fs";
import * as path from "path";
import * as readline from "readline";
import { fixArabicDisplay } from "./utils/text-fixer.js";

const program = new Command();

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

const askQuestion = (query: string): Promise<string> => {
    return new Promise((resolve) => rl.question(query, resolve));
};

program
  .name("nex-pick")
  .description("NexPicker — Interactive DOM selector picker targeting NexScrape schemas")
  .argument("[url]", "URL to open in the picker (optional, will prompt if missing)")
  .option("-o, --output-dir <dir>", "Directory to save schemas", ".nexscrape")
  .option("-s, --schema <file>", "Existing schema file to append to")
  .option("-b, --browser <name>", "Browser to use (chromium/firefox/webkit)", "chromium")
  .action(async (url, options) => {
    
    console.log(chalk.hex('#eb5e28').bold(`\n🕷️  NexPicker v0.1.0\n`));

    let targetUrl = url;
    if (!targetUrl) {
        targetUrl = await askQuestion(chalk.cyan("🌐 Please enter the URL you want to scrape: "));
    }

    if (!targetUrl || (!targetUrl.startsWith("http://") && !targetUrl.startsWith("https://"))) {
        console.error(chalk.red("\n❌ Invalid URL. Please start with http:// or https://"));
        process.exit(1);
    }
    
    const spinner = ora("Launching browser...").start();

    const config = { ...DEFAULT_PICKER_CONFIG, url: targetUrl, ...options };
    const schemaWriter = new SchemaWriter(config);
    const engine = new AnalysisEngine();
    
    const collectedFields: any[] = [];

    const session = new BrowserSession({
      config,
      onOverlayMessage: async (msg: OverlayMessage) => {
        
        switch (msg.type) {
          case "element:click":
            const spinnerAnalyze = ora(`Analyzing element...`).start();
            try {
              // 1. Analyze
              const result = await engine.analyzeElement(msg.data, session['page']!);
              spinnerAnalyze.succeed(`Analysis complete`);

              console.log(chalk.gray(`\n┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓`));
              console.log(chalk.green(`  ✅ Element Selected:`));
              console.log(chalk.gray(`  Preview:    `) + fixArabicDisplay(result.field.preview));
              console.log(chalk.blue(`  Field Name: `) + result.field.name);
              console.log(chalk.blue(`  Type:       `) + result.field.type);
              
              const primary = result.field.selector.primary;
              const displaySelector = primary.type === "text" ? fixArabicDisplay(primary.value) : primary.value;
              console.log(chalk.yellow(`\n  🎯 Best Selector (${primary.type}):`));
              console.log(`     ${chalk.bold(displaySelector)}`);
              console.log(chalk.gray(`     Stability: ${primary.stability} | Matches: ${primary.matchCount}`));
              
              const cssFallback = result.field.selector.fallbacks.find(f => f.type === 'css' && f.value.includes(' > ')) 
                               || result.field.selector.fallbacks.find(f => f.type === 'css');
              if (cssFallback) {
                  console.log(chalk.cyan(`\n  🔗 CSS Path:`));
                  console.log(`     ${chalk.bold(cssFallback.value)}`);
              }
              
              const xpathFallback = result.field.selector.fallbacks.find(f => f.type === 'xpath' && f.value.includes('/') && !f.value.startsWith('//' + result.field.selector.primary.value))
                                 || result.field.selector.fallbacks.find(f => f.type === 'xpath');
              if (xpathFallback && primary.type !== 'xpath') {
                  console.log(chalk.magenta(`\n  🔗 XPath Path:`));
                  console.log(`     ${chalk.bold(xpathFallback.value)}`);
              }
              
              if (result.field.warnings.length > 0) {
                 console.log(chalk.bgYellow.black(`\n  ⚠️ Warnings:`));
                 result.field.warnings.forEach(w => console.log(`    - ${w}`));
              }
              console.log(chalk.gray(`┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛`));

              // 3. Accumulate in Memory instead of aggressive auto-save
              collectedFields.push(result.field);
              console.log(chalk.green(`\n📥 Added to memory (${collectedFields.length} field(s) buffered)`));

            } catch (err: any) {
              spinnerAnalyze.fail(`Analysis failed: ${err.message}`);
            }
            break;
            
          case "export:save":
            try {
               const filename = msg.data.filename || "schema.json";
               const finalPath = path.resolve(process.cwd(), filename);
               
               const exportData = {
                   url: targetUrl,
                   generatedAt: new Date().toISOString(),
                   fields: collectedFields
               };

               fs.writeFileSync(finalPath, JSON.stringify(exportData, null, 2));
               console.log(chalk.bgGreen.black(`\n✅ Schema successfully saved to: ${finalPath}\n`));
            } catch(e: any) {
               console.error(chalk.bgRed(`\n❌ Failed to export schema: ${e.message}`));
            }
            break;

          case "element:hover":
            // Optional: print out live hover status, but might be too spammy in terminal
            break;
        }
      },
      onPageClose: () => {
         console.log(chalk.yellow(`\nBrowser closed. Exiting NexPicker.`));
         process.exit(0);
      }
    });

    try {
      await session.launch();
      spinner.text = "Loading page and injecting overlay...";
      await session.navigateAndInject();
      spinner.succeed(`Ready! Please select elements in the browser window.`);
      
      console.log(chalk.gray(`\nWaiting for selections... (Close browser to exit)`));
      
    } catch (e: any) {
      spinner.fail(`Failed to launch: ${e.message}`);
      process.exit(1);
    }
  });

program.parse();
