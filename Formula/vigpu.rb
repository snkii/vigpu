class Vigpu < Formula
  desc "A simple, pretty GPU monitoring TUI"
  homepage "https://seon.uk/vigpu/"
  url "https://github.com/snkii/vigpu/releases/download/v0.1.4/vigpu-macos-arm64.tar"
  sha256 "5d86fd334fa964132e5a6524528d469137feefe9158cb5c025a94d9c9df92e4f"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on arch: :arm64

  def install
    bin.install "vigpu"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/vigpu --version")
  end
end
