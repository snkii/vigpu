class Vigpu < Formula
  desc "A simple, pretty GPU monitoring TUI"
  homepage "https://seon.uk/vigpu/"
  url "https://github.com/snkii/vigpu/releases/download/v0.1.0/vigpu-macos-arm64.tar"
  sha256 "0feca776cf2ca429464e75e2e21b53c30d730d3ac392f5c08b11980780f0d5e5"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on arch: :arm64

  def install
    bin.install "vigpu"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/vigpu --version")
  end
end
