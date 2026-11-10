import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("the foundation landing page labels planned domains as planned", async () => {
  const page = await readFile(new URL("../src/app/page.tsx", import.meta.url), "utf8");
  assert.match(page, /PLANNED MODULE/);
  assert.match(page, /One core\. Many operations\./);
});

test("the web application does not include database credentials", async () => {
  const packageJson = await readFile(new URL("../package.json", import.meta.url), "utf8");
  assert.doesNotMatch(packageJson, /DATABASE_URL|postgres(?:ql)?:\/\//);
});
