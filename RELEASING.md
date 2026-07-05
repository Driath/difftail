# Releasing difftail

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
gh repo create difftail --public --source . --push
```

Tagging pushes an auto-generated source tarball at:
`https://github.com/Driath/difftail/archive/refs/tags/v0.1.0.tar.gz`

## 3. Wire the Homebrew formula

Get the tarball sha256:

```sh
curl -sL https://github.com/Driath/difftail/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
```

Then in `Formula/difftail.rb` replace `Driath`, the `url` tag, and
`REPLACE_WITH_RELEASE_TARBALL_SHA256`.

## 4. Publish the tap

```sh
gh repo create homebrew-tap --public
# copy Formula/difftail.rb into it, commit, push
```

Users install with:

```sh
brew install Driath/tap/difftail
```
