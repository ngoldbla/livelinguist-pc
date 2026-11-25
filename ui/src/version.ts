// App version shown in the UI. ui/package.json mirrors the workspace
// Cargo.toml version (checked by version.test.ts); release tags are v<version>.
import pkg from "../package.json";

export const APP_VERSION: string = pkg.version;
