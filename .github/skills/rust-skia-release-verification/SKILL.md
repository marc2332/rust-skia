---
name: rust-skia-release-verification
description: Verify a prepared rust-skia release after the release branch has built its binaries. Use to check release commits, CI, binary assets, generated images, crate packaging, and release notes before publication. This skill never publishes crates or creates GitHub releases.
---

# rust-skia Release Verification

Read the [release checklist](../../release-checklist.md) and the
[release notes rules](../../release-notes-guidelines.md). Coordinate executable
operations through the verification recipes in the repository `justfile`.

Establish the release version, previous version, exact release commit, and
canonical remote. Never proceed across a mismatch: remote `master` and
`release`, binary artifacts, and the candidate commit must identify the same
release.

1. Run `just release-verify <version> <previous-version> <commit>`. It requires
   a clean working tree with the Skia submodule at its recorded tag, and `emcc`
   for the post-release smoke test. It verifies the branches and binary
   workflows/assets, verifies binary-download and source-build packaging, and
   dry-runs `skia-svg-macros` packaging when that crate changed. Its output ends
   with a `Summary:` block that lists every check that passed.
2. QA no longer produces image artifacts. If both commits have separately
   generated artifacts, run `just release-verify-images <previous-version> <commit> <platforms>`.
   Inspect the summary and review images in `/tmp/rust-skia-release-images/comparison/`.
   Unexplained CPU image changes block publication. Optionally run
   `just release-verify-vector <previous-version> <commit> <platforms>` for an advisory SVG comparison.
   If there are no artifacts, record that image comparison was not run.
3. Write the release notes. The maintainer does not write them, so unless the
   user names an existing notes file, draft
   `$TMPDIR/rust-skia-<version>-notes.md` from the PRs merged since the
   previous tag, following the release notes rules. The intro paragraph states
   only what the release aligns to and the range it covers. End the notes with
   the repository's AI disclosure, directly above the `Full Changelog:` line:

   ```markdown
   > [!NOTE]
   > This was created with AI assistance.
   ```

   Validate the finished notes against the release notes rules.
4. Report the verified version, previous version, exact commit, notes file,
   canonical remote, and image-review result.

## Options

- Pass a quoted list of `workflow:artifact` pairs as `platforms` to either recipe.
- To compare PDF instead of SVG, pass `pdf` after `platforms` in the vector recipe.
- To pin the run instead of selecting the latest completed run with the artifact,
  pass a space-separated list of `workflow:run`
  pairs as the `runs` argument to either recipe. This is useful when a commit
  has multiple successful runs and you want a specific one.

## Failure prevention

- Run source-build, package, and documentation-binding checks serially. They
   share `skia-bindings/skia/third_party/externals`; concurrent builds can leave
   nested dependency checkouts at incompatible revisions or with missing files.
- Use `just release-verify` for package checks instead of raw
   `cargo publish --dry-run`. A source build may legitimately modify its
   extracted Skia tree, so verification packages with `--no-verify`, extracts
   the crate, and builds that exact package explicitly.
- Treat direct `skia-bindings` features as independent. Checks that enable
   `gl` or `vulkan` and exercise Ganesh must also enable `ganesh` explicitly.
- Before handing off to publication, ensure no verification process is still
   using the shared Skia checkout.

This skill ends after reporting verification. Never invoke a publishing
recipe; publication requires a separate invocation of the
`rust-skia-release-publishing` skill.

Any source fix invalidates verification. Return to release preparation, then
rerun both verification gates.
