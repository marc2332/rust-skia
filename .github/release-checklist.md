# Release Checklist

Start here only after the `release` branch is ready and its binary workflows
have finished. The release verification skill drafts the release notes. All
executable steps go through `just`.

Set `<version>`, `<previous>`, `<commit>`, and `<notes>` to concrete values.

## Verify

- [ ] Run `just release-verify <version> <previous> <commit>`. This verifies the
      release/master commit, every binary workflow, the binary release assets,
      binary downloads, source builds, and the optional `skia-svg-macros`
      package dry run.
- [ ] If both commits have separately generated image artifacts, compare them
      using `just release-verify-images <previous> <commit> <platforms>` and
      optionally `just release-verify-vector <previous> <commit> <platforms>`.
      Pass a quoted list of `workflow:artifact` pairs. QA no longer uploads images.
- [ ] Review `<notes>` against the
      [release notes guidelines](release-notes-guidelines.md).

Image comparison exits 0 for an exact match, 1 when differences need human
review, and 2 when the check could not run. Stop for failed release workflows,
missing binaries, source-build fallback, or unexplained CPU image changes if
images were compared. Vector differences are advisory.

## Publish

Crate and GitHub publication are irreversible and require explicit approval.

- [ ] Run `just release-publish-crates <version> <previous>`. It publishes only
      the required crates in dependency order, waits for each registry entry,
      and runs the post-release smoke test. If `skia-svg-macros` changed, it is
      published first.
- [ ] Run `just release-publish-github <version> <commit> <notes> true`. All
      rust-skia GitHub releases are prereleases. This creates or resumes the
      GitHub release and verifies its tag.

Never republish an existing crate version. If only `skia-svg-macros` has been
published, reuse it and resume. If the release commit changes after
`skia-bindings` or `skia-safe` is published, use a new rust-skia patch version.
