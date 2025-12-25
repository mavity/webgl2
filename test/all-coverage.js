import { before, after } from 'node:test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Enable debug/coverage mode for all tests in this process
process.env.WEBGL2_DEBUG = 'true';

import { webGL2 } from '../index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const gl = await webGL2({ debug: true });

after(async () => {
  // 1. Parse current run coverage
  const currentCoverage = {};
  const lcov = gl.getLcovReport();
  
  if (lcov) {
    let currentFile = null;
    for (const line of lcov.split('\n')) {
      if (line.startsWith('SF:')) {
        currentFile = line.substring(3);
        if (!currentCoverage[currentFile]) currentCoverage[currentFile] = {};
      } else if (line.startsWith('DA:')) {
        const [lineNum, hits] = line.substring(3).split(',').map(Number);
        if (currentFile) {
          currentCoverage[currentFile][lineNum] = (currentCoverage[currentFile][lineNum] || 0) + hits;
        }
      }
    }
  }

  // 2. Handle Aggregation
  const TARGET_DIR = path.join(__dirname, '..', 'target', 'coverage');
  const RAW_COVERAGE_PATH = path.join(TARGET_DIR, 'raw_coverage.json');
  const LOCK_DIR = path.join(TARGET_DIR, 'coverage.lock');
  
  if (!fs.existsSync(TARGET_DIR)) {
    fs.mkdirSync(TARGET_DIR, { recursive: true });
  }

  // Simple spin-lock using mkdir to ensure atomic read-modify-write
  const acquireLock = () => {
    try {
      fs.mkdirSync(LOCK_DIR);
      return true;
    } catch (e) {
      return false;
    }
  };

  const releaseLock = () => {
    try {
      fs.rmdirSync(LOCK_DIR);
    } catch (e) {}
  };

  const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

  let locked = false;
  // Try to acquire lock for up to 5 seconds
  for (let i = 0; i < 100; i++) {
    if (acquireLock()) {
      locked = true;
      break;
    }
    await sleep(50);
  }

  if (!locked) {
    console.error(`[Coverage] Failed to acquire lock for ${process.pid}, skipping coverage update.`);
    return;
  }

  try {
    let aggregatedData = {};
    const currentPpid = process.ppid;
    
    // console.log(`[Coverage] Process ${process.pid} (ppid ${currentPpid}) saving coverage...`);

    // Try to load existing coverage
    if (fs.existsSync(RAW_COVERAGE_PATH)) {
      try {
        const stored = JSON.parse(fs.readFileSync(RAW_COVERAGE_PATH, 'utf8'));
        // Only aggregate if it belongs to the same test run (same parent PID)
        if (stored.ppid === currentPpid) {
          aggregatedData = stored.data;
          // console.log(`[Coverage] Loaded existing coverage for ppid ${currentPpid}. Keys: ${Object.keys(aggregatedData).length}`);
        } else {
          // console.log(`[Coverage] New run detected (ppid ${currentPpid} != ${stored.ppid}), resetting coverage.`);
        }
      } catch (e) {
        // ignore error, proceed with reset
      }
    }

    // Merge current into aggregated
    let newLines = 0;
    for (const file in currentCoverage) {
      if (!aggregatedData[file]) {
        aggregatedData[file] = {};
      }
      for (const line in currentCoverage[file]) {
        if (!aggregatedData[file][line]) newLines++;
        aggregatedData[file][line] = (aggregatedData[file][line] || 0) + currentCoverage[file][line];
      }
    }
    // console.log(`[Coverage] Merged ${Object.keys(currentCoverage).length} files. New lines covered: ${newLines}`);

    // Save updated raw coverage
    fs.writeFileSync(RAW_COVERAGE_PATH, JSON.stringify({
      ppid: currentPpid,
      data: aggregatedData
    }, null, 2));

    // 3. Generate Markdown from Aggregated Data
    let output = '# Coverage Report\n\n';
    output += '| File | Lines Covered | Lines Missed | Total Lines | Coverage |\n';
    output += '|---|---|---|---|---|\n';

    let totalCovered = 0;
    let totalInstrumented = 0;

    const getEmoji = (pct) => {
      if (pct >= 80) return '🟢';
      if (pct >= 50) return '🟡';
      return '🔴';
    };

    for (const file of Object.keys(aggregatedData).sort()) {
      if (!file.startsWith('src/')) continue;
      const lines = aggregatedData[file];
      const totalLines = Object.keys(lines).length;
      const coveredLines = Object.values(lines).filter(h => h > 0).length;
      const missedLines = totalLines - coveredLines;
      const pctVal = totalLines ? (coveredLines / totalLines) * 100 : 0;
      const pct = pctVal.toFixed(2);
      const emoji = getEmoji(pctVal);
      output += `| ${file} | ${coveredLines} | ${missedLines} | ${totalLines} | ${pct}% ${emoji} |\n`;
            
      totalCovered += coveredLines;
      totalInstrumented += totalLines;
    }

    const totalMissed = totalInstrumented - totalCovered;
    const totalPctVal = totalInstrumented ? (totalCovered / totalInstrumented) * 100 : 0;
    const totalPct = totalPctVal.toFixed(2);
    const totalEmoji = getEmoji(totalPctVal);
    output += `| **Total** | **${totalCovered}** | **${totalMissed}** | **${totalInstrumented}** | **${totalPct}% ${totalEmoji}** |\n`;

    const outputPath = path.join(__dirname, '..', 'coverage.md');
    fs.writeFileSync(outputPath, output);

  } finally {
    releaseLock();
  }
});