---
title: "VS Code extension missing `build` script — `npm run build` is absent"
labels: ["build", "extension", "good first issue"]
assignees: []
---

## Summary

The repository's build script (`scripts/build-windows.ps1`) attempts to run `npm run build` in the `vscode-extension` folder, but the extension's `package.json` does not define a `build` script. This causes the CI/local build to report `Missing script: "build"` (the build script currently continues, but it's a missing step).

## Reproduction

1. Run `scripts\build-windows.ps1` on Windows (or equivalent steps):
   - The script runs `npm install` in `vscode-extension` then attempts `npm run build`.
2. Observe the npm error: `Missing script: "build"` in the logs.

## Suggested fix

Add a `build` script to `vscode-extension/package.json`. Example minimal options:

1. If the extension has a TypeScript build step, add a script that compiles it:

```json
"scripts": {
  "build": "tsc -p ./"
}
```

2. If the extension doesn't need a build step (no compile step), add a no-op `build` script so the repo-level build is deterministic:

```json
"scripts": {
  "build": "echo \"No build step required for vscode-extension\""
}
```

3. Alternatively, update `scripts/build-windows.ps1` to skip `npm run build` when the script is missing (current behavior).

## Acceptance criteria

- `npm run build` in `vscode-extension` either succeeds or is intentionally skipped and documented.
- CI and local builds do not report `Missing script: "build"` as an unexpected error.

## Notes

- I attempted to open this issue via the GitHub CLI but `gh` is not installed on the host; creating this issue file under `.github/ISSUES/` as a fallback. If you want, I can open it on GitHub directly when `gh` is available or if you provide a token.
