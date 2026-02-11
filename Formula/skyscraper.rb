class Skyscraper < Formula
  desc "A TUI client for Bluesky"
  homepage "https://github.com/cameronbanga/skyscraper-cli"
  version "0.1.0"

  on_macos do
    url "https://github.com/cameronbanga/skyscraper-cli/releases/download/v#{version}/skyscraper-#{version}-universal-apple-darwin.tar.gz"
    # sha256 "UPDATE_WITH_ACTUAL_SHA256"
  end

  def install
    bin.install "skyscraper"
  end

  test do
    assert_match "skyscraper", shell_output("#{bin}/skyscraper --version")
  end
end
