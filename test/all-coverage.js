import { before, after } from 'node:test';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

import { webGL2 } from '../index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const gl = await webGL2({ debug: true });

after(async () => {
  const lcov = gl.getLcovReport();
  // Simple LCOV to Markdown conversion
  let output = '# Coverage Report\n\n';
  output += '| File | Lines Covered | Lines Missed | Total Lines | Coverage |\n';
  output += '|---|---|---|---|---|\n';

  const coverage = {};
  let currentFile = null;
  for (const line of lcov.split('\n')) {
    if (line.startsWith('SF:')) {
      currentFile = line.substring(3);
      if (!coverage[currentFile]) coverage[currentFile] = {};
    } else if (line.startsWith('DA:')) {
      const [lineNum, hits] = line.substring(3).split(',').map(Number);
      if (currentFile) {
        coverage[currentFile][lineNum] = (coverage[currentFile][lineNum] || 0) + hits;
      }
    }
  }

  let totalCovered = 0;
  let totalInstrumented = 0;

  const getEmoji = (pct) => {
    if (pct >= 80) return '🟢';
    if (pct >= 50) return '🟡';
    return '🔴';
  };

  for (const file of Object.keys(coverage).sort()) {
    if (!file.startsWith('src/')) continue;
    const lines = coverage[file];
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
});