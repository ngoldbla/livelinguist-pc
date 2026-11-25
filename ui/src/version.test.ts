import { expect, test } from "bun:test";
import { APP_VERSION } from "./version";

test("UI version matches the workspace Cargo.toml version", async () => {
  const cargo = await Bun.file(`${import.meta.dir}/../../Cargo.toml`).text();
  const match = cargo.match(/\[workspace\.package\][^[]*?\nversion\s*=\s*"([^"]+)"/);
  expect(match?.[1]).toBe(APP_VERSION);
});
