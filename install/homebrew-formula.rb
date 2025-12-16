# Homebrew Formula for DevHub
#
# To create your own Homebrew tap:
# 1. Create a GitHub repo: moinsen-dev/homebrew-tap
# 2. Add this file as Formula/devhub.rb
# 3. Users can then: brew tap moinsen-dev/tap && brew install devhub
#
# Update the sha256 and url after each release.

class Devhub < Formula
  desc "Multi-project development environment manager"
  homepage "https://github.com/moinsen-dev/devhub"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/moinsen-dev/devhub/releases/download/v#{version}/devhub-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
    else
      url "https://github.com/moinsen-dev/devhub/releases/download/v#{version}/devhub-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/moinsen-dev/devhub/releases/download/v#{version}/devhub-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
    else
      url "https://github.com/moinsen-dev/devhub/releases/download/v#{version}/devhub-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
    end
  end

  def install
    bin.install "devhub"

    # Generate shell completions
    generate_completions_from_executable(bin/"devhub", "completions")
  end

  # Optional: Install LaunchAgent for daemon auto-start
  service do
    run [opt_bin/"devhub", "daemon"]
    keep_alive true
    log_path var/"log/devhub.log"
    error_log_path var/"log/devhub.error.log"
  end

  test do
    assert_match "devhub #{version}", shell_output("#{bin}/devhub --version")
  end
end
