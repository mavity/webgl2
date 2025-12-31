import { before, after } from 'node:test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// Enable debug/coverage mode for all tests in this process
process.env.WEBGL2_DEBUG = 'true';

import { webGL2, debug } from '../index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const gl = await webGL2({ debug: true });

after(async () => {
  // 1. Parse current run coverage
  const currentCoverage = {};
  const lcov = debug.getLcovReport(gl);
  
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

  // 2. Handle Aggregation (Lock-Free)
  const TARGET_DIR = path.join(__dirname, '..', 'target', 'coverage');
  const RAW_COVERAGE_PATH = path.join(TARGET_DIR, 'raw_coverage.json');
  
  if (!fs.existsSync(TARGET_DIR)) {
    fs.mkdirSync(TARGET_DIR, { recursive: true });
  }

  let aggregatedData = null;
  const myStartTime = Date.now() - (process.uptime() * 1000);
  const MAX_RETRIES = 30;

  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    // 2. Read & Assess
    let baseData = {};
    let baseTimestamp = Date.now();
    let fileContent = null;

    if (fs.existsSync(RAW_COVERAGE_PATH)) {
      try {
        fileContent = fs.readFileSync(RAW_COVERAGE_PATH, 'utf8');
        const stored = JSON.parse(fileContent);
        
        if (stored.ppid === process.ppid) {
          // Case B: PPID Match
          baseData = stored.data;
          baseTimestamp = stored.timestamp;
        } else {
          // Case C: PPID Mismatch
          if (stored.timestamp > myStartTime) {
            // Newer run detected. Bail.
            console.log('[Coverage] Newer run detected. Bailing out.');
            return;
          }
          // Older run. Overwrite.
          baseData = {};
          baseTimestamp = Date.now();
        }
      } catch (e) {
        // Corrupted or empty. Treat as new.
        baseData = {};
        baseTimestamp = Date.now();
      }
    } else {
      // Case A: File Missing
      baseData = {};
      baseTimestamp = Date.now();
    }

    // 3. Merge
    const mergedData = JSON.parse(JSON.stringify(baseData));
    for (const file in currentCoverage) {
      if (!mergedData[file]) mergedData[file] = {};
      for (const line in currentCoverage[file]) {
        mergedData[file][line] = (mergedData[file][line] || 0) + currentCoverage[file][line];
      }
    }

    // 4. Pre-Write Check
    if (fs.existsSync(RAW_COVERAGE_PATH)) {
      const currentRaw = fs.readFileSync(RAW_COVERAGE_PATH, 'utf8');
      if (fileContent !== null && currentRaw !== fileContent) {
        await new Promise(r => setTimeout(r, Math.random() * 300 + 100));
        continue;
      }
      if (fileContent === null) {
        await new Promise(r => setTimeout(r, Math.random() * 300 + 100));
        continue;
      }
    } else if (fileContent !== null) {
      await new Promise(r => setTimeout(r, Math.random() * 300 + 100));
      continue;
    }

    // 5. Write
    const writeData = {
      ppid: process.ppid,
      timestamp: baseTimestamp,
      data: mergedData
    };
    fs.writeFileSync(RAW_COVERAGE_PATH, JSON.stringify(writeData, null, 2));

    // 6. Delay
    await new Promise(r => setTimeout(r, Math.random() * 300 + 100));

    // 7. Verify
    try {
      const verifyRaw = fs.readFileSync(RAW_COVERAGE_PATH, 'utf8');
      const verifyData = JSON.parse(verifyRaw);

      if (verifyData.ppid !== process.ppid) {
        console.log('[Coverage] Another run took over. Bailing out.');
        return;
      }
      if (verifyData.timestamp !== baseTimestamp) {
        console.log('[Coverage] Timestamp mismatch. Bailing out.');
        return;
      }

      let semanticCheckPassed = true;
      for (const file in currentCoverage) {
        if (!verifyData.data[file]) {
          semanticCheckPassed = false;
          break;
        }
        for (const line in currentCoverage[file]) {
          if ((verifyData.data[file][line] || 0) < currentCoverage[file][line]) {
            semanticCheckPassed = false;
            break;
          }
        }
      }

      if (semanticCheckPassed) {
        aggregatedData = verifyData.data;
        break;
      }
    } catch (e) {
      // Retry
    }
  }

  if (!aggregatedData) {
    console.error('[Coverage] Failed to update coverage after retries.');
    return;
  }

  // 3. Generate Markdown from Aggregated Data
  let output = '# Coverage Report\n\n';
  output += '> **Legend:** 🟢 ≥80% | 🟡 ≥50% (or ≤6 lines) | 🟠 ≥20% | 🔴 <20%\n\n';
  output += '| File | Lines Covered | Lines Missed | Total Lines | Coverage |\n';
  output += '|---|---|---|---|---:|\n';

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
  output += '|---|---|---|---:|\n';

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
});
