# Releasing chrono-diff

## 1. Cut a version

```sh
# bump `version` in Cargo.toml, then:
git commit -am "release: v0.1.0"
git tag v0.1.0
git push origin main --tags
```

## 2. Publish on GitHub

Create a GitHub repo (once) and push:

```sh
gh repo create chrono-diff --public --source . --push
```

Tagging pushes an auto-generated source tarball at:
`https://github.com/<you>/chrono-diff/archive/refs/tags/v0.1.0.tar.gz`

## 3. Wire the Homebrew formula

Get the tarball sha256:

```sh
curl -sL https://github.com/<you>/chrono-diff/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
```

Then in `Formula/chrono-diff.rb` replace `<you>`, the `url` tag, and
`REPLACE_WITH_RELEASE_TARBALL_SHA256`.

## 4. Publish the tap

```sh
gh repo create homebrew-tap --public
# copy Formula/chrono-diff.rb into it, commit, push
```

Users install with:

```sh
brew install <you>/tap/chrono-diff
```
