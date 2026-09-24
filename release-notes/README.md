# Release notes ("What's new")

User-facing notes for every toolchain release, rendered by the website's
"What's new" pages. The contract is `docs/release-notes-schema.md` in
`xiom-lang/website` (schema v1); this directory is the authoring side.

## Authoring (before every tag)

1. Copy `TEMPLATE.md` to `release-notes/<tag>.md` (for example
   `release-notes/v0.62.0.md`) and fill it in:
   * one summary sentence (no version number, no internal identifiers);
   * 1-6 highlights, each with a `kind:` line (`language`, `compiler`,
     `stdlib`, `tooling`, `fix`, `security`);
   * a `## Breaking changes` section -- write `- None.` when there is
     nothing, because that is the only way to say it;
   * optional `## Known issues` and `## Docs` entries.
   Plain ASCII only, no markdown, HTML, emoji, task ids, wave names or
   commit hashes. Write for a user deciding whether to upgrade.
2. The stdlib repository contributes `release-notes/<tag>.md` at the ref
   pinned by `STDLIB_VERSION`; its highlights are merged automatically and
   default to `kind: stdlib` unless the fragment sets its own kind. The
   compiler document's summary is the release summary.
3. Convert and commit the JSON **before creating the tag**:

   ```
   cargo run --quiet -p xiom-release-notes -- convert --tag v0.62.0 --stdlib ../stdlib
   git add release-notes/v0.62.0.md release-notes/v0.62.0.json
   ```

   The tag-pinned raw file
   (`https://raw.githubusercontent.com/xiom-lang/xiom/<tag>/release-notes/<tag>.json`)
   is the website's fallback source, so it must exist at the tag.

## Release gate

The release workflow runs, before `gh release create`:

```
cargo run --quiet -p xiom-release-notes -- verify --tag "$GITHUB_REF_NAME" --stdlib ../stdlib
```

`verify` regenerates the document from the markdown sources and fails unless
the committed JSON matches byte for byte -- a release cannot ship notes that
drift from their markdown, and schema violations (missing summary, over-long
fields, unknown kinds, absent `breaking` array, internal identifiers) fail
hard.

## Publishing

The website lane publishes the same committed file to
`https://dl.xiom-lang.org/releases/<tag>/release.json` and adds
`"notes": true` to that tag in `releases/index.json`; the
`compiler-release` dispatch payload carries `notes_path` for it. No website
deploy is needed: the pages fetch notes by tag at runtime.
