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
    
    // Try to load existing coverage
    if (fs.existsSync(RAW_COVERAGE_PATH)) {
      try {
        const stored = JSON.parse(fs.readFileSync(RAW_COVERAGE_PATH, 'utf8'));
        // Only aggregate if it belongs to the same test run (same parent PID)
        if (stored.ppid === currentPpid) {
          aggregatedData = stored.data;
        }
      } catch (e) {
        // ignore error, proceed with reset
      }
    }

    // Merge current into aggregated
    for (const file in currentCoverage) {
      if (!aggregatedData[file]) {
        aggregatedData[file] = {};
      }
      for (const line in currentCoverage[file]) {
        aggregatedData[file][line] = (aggregatedData[file][line] || 0) + currentCoverage[file][line];
      }
    }

    // Save updated raw coverage
    fs.writeFileSync(RAW_COVERAGE_PATH, JSON.stringify({
      ppid: currentPpid,
      data: aggregatedData
    }, null, 2));

    // 3. Generate Markdown from Aggregated Data
    let output = '# Coverage Report\n\n';
    output += '> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%\n\n';
    output += '| File | Lines Covered | Lines Missed | Total Lines | Coverage |\n';
    output += '|---|---|---|---|---|\n';

    let totalCovered = 0;
    let totalInstrumented = 0;
    const fileStats = [];

    const getEmoji = (pct, totalLines) => {
      if (pct >= 80) return '🟢';
      if (totalLines > 0 && totalLines <= 6) return '🟡';
      if (pct >= 50) return '🟡';
      if (pct >= 20) return '🟠';
      return '🔴';
    };

    for (const file of Object.keys(aggregatedData).sort()) {
      if (!file.startsWith('src/')) continue;

      // Filter out files that don't exist locally (e.g. dependencies)
      if (!fs.existsSync(path.join(__dirname, '..', file))) continue;

      const lines = aggregatedData[file];
      const totalLines = Object.keys(lines).length;
      const coveredLines = Object.values(lines).filter(h => h > 0).length;
      const missedLines = totalLines - coveredLines;
      const pctVal = totalLines ? (coveredLines / totalLines) * 100 : 0;
      
      fileStats.push({
        file,
        missedLines,
        totalLines,
        linesObj: lines,
        pctVal
      });

      const pct = pctVal.toFixed(2);
      const emoji = getEmoji(pctVal, totalLines);
      output += `| ${file} | ${coveredLines} | ${missedLines} | ${totalLines} | ${pct}% ${emoji} |\n`;
            
      totalCovered += coveredLines;
      totalInstrumented += totalLines;
    }

    const totalMissed = totalInstrumented - totalCovered;
    const totalPctVal = totalInstrumented ? (totalCovered / totalInstrumented) * 100 : 0;
    const totalPct = totalPctVal.toFixed(2);
    const totalEmoji = getEmoji(totalPctVal, totalInstrumented);
    output += `| **Total** | **${totalCovered}** | **${totalMissed}** | **${totalInstrumented}** | **${totalPct}% ${totalEmoji}** |\n`;

    // 4. Top Missed Files Table
    output += '\n## Top Missed Files\n\n';
    output += '| File | Lines Missed | Illustrative Line | Coverage |\n';
    output += '|---|---|---|---|\n';

    const topMissed = fileStats.sort((a, b) => b.missedLines - a.missedLines).slice(0, 4);

    for (const stat of topMissed) {
      if (stat.missedLines === 0) continue;

      const missedLineNums = Object.keys(stat.linesObj)
        .filter(line => stat.linesObj[line] === 0)
        .map(Number)
        .sort((a, b) => a - b);
      
      let maxSpan = { start: -1, end: -1, len: 0 };
      let currentSpan = { start: -1, end: -1, len: 0 };

      for (const line of missedLineNums) {
        if (currentSpan.start === -1) {
          currentSpan = { start: line, end: line, len: 1 };
        } else if (line === currentSpan.end + 1) {
          currentSpan.end = line;
          currentSpan.len++;
        } else {
          if (currentSpan.len > maxSpan.len) maxSpan = currentSpan;
          currentSpan = { start: line, end: line, len: 1 };
        }
      }
      if (currentSpan.len > maxSpan.len) maxSpan = currentSpan;

      let illustrativeCode = '';
      try {
        const filePath = path.join(__dirname, '..', stat.file);
        if (fs.existsSync(filePath)) {
            const content = fs.readFileSync(filePath, 'utf8').split('\n');
            
            let bestLine = null;
            let minIndent = Infinity;

            for (let i = maxSpan.start; i <= maxSpan.end; i++) {
                const lineContent = content[i - 1];
                if (!lineContent || !lineContent.trim()) continue;
                
                const indent = lineContent.search(/\S/);
                if (indent !== -1 && indent < minIndent) {
                    minIndent = indent;
                    bestLine = { num: i, text: lineContent.trim() };
                }
            }
            
            if (bestLine) {
                let codeText = bestLine.text;
                if (codeText.length > 60) codeText = codeText.substring(0, 57) + '...';
                // Escape backticks for markdown
                codeText = codeText.replace(/`/g, '\\`');
                illustrativeCode = `[${bestLine.num}] \`${codeText}\``;
            } else {
                illustrativeCode = '(No content)';
            }
        } else {
            illustrativeCode = '(File not found)';
        }
      } catch (e) {
        illustrativeCode = '(Error reading file)';
      }

      const pct = stat.pctVal.toFixed(2);
      const emoji = getEmoji(stat.pctVal, stat.totalLines);
      output += `| ${stat.file} | ${stat.missedLines}/${stat.totalLines} | ${illustrativeCode} | ${pct}% ${emoji} |\n`;
    }

    const outputPath = path.join(__dirname, '..', 'coverage.md');
    fs.writeFileSync(outputPath, output);

  } finally {
    releaseLock();
  }
});
