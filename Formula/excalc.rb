class Excalc < Formula
  desc "Expression calculator CLI tool and MCP server for AI coding agents"
  homepage "https://github.com/dblock/excalc-rs"
  url "https://github.com/dblock/excalc-rs/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "faf98587b4e5bea66aa7e685f3906da4da90bec5c0df8d7cc34be6256770f15a"
  license "MIT"
  head "https://github.com/dblock/excalc-rs.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_equal "4", shell_output("#{bin}/excalc '2 + 2'").strip
    assert_equal "4", shell_output("#{bin}/calc '2 + 2'").strip
    assert_path_exists bin/"excalc-mcp"
  end
end
