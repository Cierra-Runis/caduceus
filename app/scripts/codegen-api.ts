/**
 * Generates Zod schemas from the server's OpenAPI spec (docs/openapi.json,
 * written by `cargo run --bin gen_openapi` in server/).
 *
 *   pnpm codegen:api          regenerate src/lib/api/generated/schemas.ts
 *   pnpm codegen:api --check  fail (exit 1) if the checked-in file is stale
 *
 * The spec is plain JSON with only internal $refs, so no $ref resolution
 * (swagger-parser) is needed before handing it to the generator.
 */
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { generateZodClientFromOpenAPI } from 'openapi-zod-client';

const appDir = join(dirname(fileURLToPath(import.meta.url)), '..');
const specPath = join(appDir, '..', 'docs', 'openapi.json');
const templatePath = join(appDir, 'scripts', 'api-schemas.hbs');
const outPath = join(appDir, 'src', 'lib', 'api', 'generated', 'schemas.ts');

const openApiDoc = JSON.parse(readFileSync(specPath, 'utf8'));

const generated = await generateZodClientFromOpenAPI({
  disableWriteToFile: true,
  openApiDoc,
  options: { shouldExportAllSchemas: true },
  templatePath,
});

if (process.argv.includes('--check')) {
  let checkedIn = '';
  try {
    checkedIn = readFileSync(outPath, 'utf8');
  } catch {
    // missing file counts as stale
  }
  if (checkedIn !== generated) {
    console.error(
      `${outPath} is stale relative to docs/openapi.json — run \`pnpm codegen:api\` and commit the result.`,
    );
    process.exit(1);
  }
  console.log('generated API schemas are up to date');
} else {
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, generated);
  console.log(`wrote ${outPath}`);
}
