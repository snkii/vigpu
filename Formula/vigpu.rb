class Vigpu < Formula
  desc "A simple, pretty GPU monitoring TUI"
  homepage "https://seon.uk/vigpu/"
  url "https://github.com/snkii/vigpu/releases/download/v0.1.2/vigpu-macos-arm64.tar"
  sha256 "a05d8594a61a23ab29c612e0333210cfc304463bf686bc80ce5c664787c110b2"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on arch: :arm64

  def install
    bin.install "vigpu"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/vigpu --version")
  end
end
