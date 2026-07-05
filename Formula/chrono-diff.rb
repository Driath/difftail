# Homebrew formula for chrono-diff.
#
# To publish a tap:
#   1. Push chrono-diff to GitHub and tag a release (see RELEASING.md).
#   2. Create a `homebrew-tap` repo and copy this file to `Formula/chrono-diff.rb`.
#   3. Replace `Driath`, the `url` tag, and the `sha256` (RELEASING.md shows how).
#   4. Users then run: `brew install Driath/tap/chrono-diff`.
#
# Until a release exists you can still install the tip:
#   brew install --HEAD Driath/tap/chrono-diff
class ChronoDiff < Formula
  desc "Real-time review feed: prints each repo change inline as agents edit"
  homepage "https://github.com/Driath/chrono-diff"
  url "https://github.com/Driath/chrono-diff/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license "MIT"
  head "https://github.com/Driath/chrono-diff.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "chrono-diff", shell_output("#{bin}/chrono-diff --version")
  end
end
