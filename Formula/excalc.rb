class Excalc < Formula
  desc "Expression calculator CLI tool and MCP server for AI coding agents"
  homepage "https://github.com/dblock/excalc-rs"
  url "https://github.com/dblock/excalc-rs/archive/refs/tags/v0.3.0.tar.gz"
  sha256 "f2258c802947d6f55043c364f7c587541389ae10a68968af2c68f1d84258622f"
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
