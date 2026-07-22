import { gzipSync } from "node:zlib";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const budgetBytes = 350 * 1024;
const assetsDirectory = path.resolve("dist/assets");

let entries;
try {
  entries = await readdir(assetsDirectory, { withFileTypes: true });
} catch (error) {
  console.error(`Bundle check failed: cannot read ${assetsDirectory}.`, error);
  process.exit(1);
}

const javascriptFiles = entries
  .filter((entry) => entry.isFile() && entry.name.endsWith(".js"))
  .map((entry) => entry.name)
  .sort();

if (javascriptFiles.length === 0) {
  console.error(`Bundle check failed: no JavaScript files found in ${assetsDirectory}.`);
  process.exit(1);
}

let totalGzipBytes = 0;
for (const fileName of javascriptFiles) {
  const source = await readFile(path.join(assetsDirectory, fileName));
  const gzipBytes = gzipSync(source).byteLength;
  totalGzipBytes += gzipBytes;
  console.log(`${fileName}: ${(gzipBytes / 1024).toFixed(2)} KB gzip`);
}

const totalKilobytes = totalGzipBytes / 1024;
const budgetKilobytes = budgetBytes / 1024;
console.log(
  `Total JavaScript: ${totalKilobytes.toFixed(2)} KB gzip ` +
    `(budget: ${budgetKilobytes.toFixed(0)} KB)`,
);

if (totalGzipBytes > budgetBytes) {
  console.error(
    `Bundle budget exceeded by ${((totalGzipBytes - budgetBytes) / 1024).toFixed(2)} KB.`,
  );
  process.exit(1);
}
