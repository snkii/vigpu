class Vigpu < Formula
  desc "A simple, pretty GPU monitoring TUI"
  homepage "https://seon.uk/vigpu/"
  url "https://github.com/snkii/vigpu/releases/download/v0.1.3/vigpu-macos-arm64.tar"
  sha256 "3267a80e9161028716c0fc72e20e30eb9fa29d92442a7ad82147fdd0ee3f00c3"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on arch: :arm64

  def install
    bin.install "vigpu"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/vigpu --version")
  end
end
