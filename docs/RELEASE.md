# Release Runbook

How to cut a new release of `mccm` and publish it to the Homebrew tap.

## Quick summary

1. Bump version in `Cargo.toml` and any README install snippets
2. Commit, tag `vX.Y.Z`, push branch + tag
3. CI builds binaries and creates the GitHub Release automatically
4. Manually push the formula to `jjroush/homebrew-tap` (auto-step is broken — see [Known issue](#known-issue-homebrew_tap_token-not-set))

## Step-by-step

### 1. Bump version

```bash
# Edit Cargo.toml
#   version = "X.Y.Z"
# Edit README.md install snippets:
#   replace the old vX.Y.Z in the curl URLs
```

Rebuild to refresh `Cargo.lock`:

```bash
cargo build
```

### 2. Commit and tag

Commit-message style matches the existing log: `vX.Y.Z: <one-line summary>`.

```bash
git add Cargo.toml Cargo.lock README.md <any-other-files>
git commit -m "vX.Y.Z: <summary>"
git tag vX.Y.Z
git push origin master
git push origin vX.Y.Z
```

The tag push triggers `.github/workflows/release.yml`.

### 3. Wait for CI

```bash
gh run watch
```

Three jobs run:
- `build` (matrix: arm64 + x86_64 macOS) — should succeed
- `release` — uploads `.tar.gz` and `.sha256` files to the GitHub Release
- `update-homebrew` — **currently fails** (see below)

Confirm the GitHub Release has artifacts:

```bash
gh release view vX.Y.Z
```

You should see four assets: arm64 + x86_64 `.tar.gz` and matching `.sha256` files.

### 4. Push formula to the Homebrew tap

Download the `.sha256` files, generate the formula from the template, and PUT it to `jjroush/homebrew-tap`:

```bash
VERSION=X.Y.Z

gh release download v${VERSION} --repo jjroush/mccm \
  --pattern "*.sha256" -D /tmp/mccm-v${VERSION}/ --clobber

SHA_ARM64=$(awk '{print $1}' /tmp/mccm-v${VERSION}/mccm-v${VERSION}-aarch64-apple-darwin.tar.gz.sha256)
SHA_X86_64=$(awk '{print $1}' /tmp/mccm-v${VERSION}/mccm-v${VERSION}-x86_64-apple-darwin.tar.gz.sha256)

sed -e "s/VERSION/${VERSION}/g" \
    -e "s/SHA256_ARM64/${SHA_ARM64}/g" \
    -e "s/SHA256_X86_64/${SHA_X86_64}/g" \
    homebrew/mccm.rb.template > /tmp/mccm.rb

SHA=$(gh api repos/jjroush/homebrew-tap/contents/Formula/mccm.rb --jq '.sha')
FORMULA=$(base64 -i /tmp/mccm.rb)

gh api -X PUT repos/jjroush/homebrew-tap/contents/Formula/mccm.rb \
  -f message="Update mccm to ${VERSION}" \
  -f content="$FORMULA" \
  -f sha="$SHA"
```

### 5. Verify

```bash
brew update
brew info jjroush/tap/mccm  # should show new version
brew upgrade jjroush/tap/mccm
mccm --version
```

## Known issue: `HOMEBREW_TAP_TOKEN` not set

The `update-homebrew` job in `.github/workflows/release.yml` tries to push the
formula automatically but fails because the `HOMEBREW_TAP_TOKEN` secret is not
configured on this repo. Symptom in the failed job log:

```
gh: To use GitHub CLI in a GitHub Actions workflow, set the GH_TOKEN environment variable.
```

The `build` and `release` jobs are unaffected — only the auto-tap-update fails.
Manual step 4 above is the workaround.

### To fix permanently

1. Create a fine-grained PAT with `Contents: read/write` on `jjroush/homebrew-tap`:
   - https://github.com/settings/personal-access-tokens/new
   - Resource owner: `jjroush`
   - Repository access: `Only select repositories` → `homebrew-tap`
   - Permissions: Repository → Contents → Read and write
2. Set it as a repo secret:
   ```bash
   gh secret set HOMEBREW_TAP_TOKEN --repo jjroush/mccm
   # paste the PAT when prompted
   ```
3. Re-run the failed `update-homebrew` job (or just wait for the next release):
   ```bash
   gh run rerun --job <job-id-from-gh-run-view>
   ```
