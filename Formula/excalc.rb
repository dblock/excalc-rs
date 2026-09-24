class Excalc < Formula
  desc "An expression calculator CLI tool and MCP server built for AI coding agents to outsource arithmetic."
  homepage "https://github.com/dblock/excalc-rs"
  url "https://github.com/dblock/excalc-rs/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "c5d0c18c85fbc45ad6091200b5d803c9248350db6e236a3d4d47b73cd3e058ea"
  license "MIT"
  head "https://github.com/dblock/excalc-rs.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_equal "4", shell_output("#{bin}/excalc '2 + 2'").strip
  end
end
