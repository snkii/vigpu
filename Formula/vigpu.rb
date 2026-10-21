class Vigpu < Formula
  desc "A simple, pretty GPU monitoring TUI"
  homepage "https://seon.uk/vigpu/"
  url "https://github.com/snkii/vigpu/releases/download/v0.1.1/vigpu-macos-arm64.tar"
  sha256 "a80849436814409abf72484fc8932d15ea2ef8ead60b5786962a4cc02fe31ef4"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on arch: :arm64

  def install
    bin.install "vigpu"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/vigpu --version")
  end
end
