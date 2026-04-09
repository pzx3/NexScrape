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

const program = new Command();

program
  .name("nex-pick")
  .description("NexPicker — Interactive DOM selector picker targeting NexScrape schemas")
  .argument("<url>", "URL to open in the picker")
  .option("-o, --output-dir <dir>", "Directory to save schemas", ".nexscrape")
  .option("-s, --schema <file>", "Existing schema file to append to")
  .option("-b, --browser <name>", "Browser to use (chromium/firefox/webkit)", "chromium")
  .action(async (url, options) => {
    
    console.log(chalk.hex('#eb5e28').bold(`\n🕷️  NexPicker v0.1.0\n`));
    const spinner = ora("Launching browser...").start();

    const config = { ...DEFAULT_PICKER_CONFIG, url, ...options };
    const schemaWriter = new SchemaWriter(config);
    const engine = new AnalysisEngine();

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

              // 2. Display results to terminal
              console.log(chalk.green(`\n✅ Element Selected!`));
              console.log(chalk.gray(`Preview:    `) + result.field.preview);
              console.log(chalk.blue(`Field Name: `) + result.field.name);
              console.log(chalk.blue(`Type:       `) + result.field.type);
              
              const primary = result.field.selector.primary;
              console.log(chalk.yellow(`\n🎯 Best Selector:`));
              console.log(`   ${chalk.bold(primary.value)}`);
              console.log(chalk.gray(`   Stability: ${primary.stability} | Matches: ${primary.matchCount}`));
              
              if (result.field.warnings.length > 0) {
                 console.log(chalk.bgYellow.black(`\n⚠️ Warnings:`));
                 result.field.warnings.forEach(w => console.log(`  - ${w}`));
              }

              // 3. Save
              if (config.autoSave) {
                 schemaWriter.writeField(result.field);
              }

            } catch (err: any) {
              spinnerAnalyze.fail(`Analysis failed: ${err.message}`);
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
