class Excalc < Formula
  desc "Expression calculator CLI tool and MCP server for AI coding agents"
  homepage "https://github.com/dblock/excalc-rs"
  url "https://github.com/dblock/excalc-rs/archive/refs/tags/v0.3.3.tar.gz"
  sha256 "659ceda0a08cc3ed19f0010ef24b4efcb7ee1c134652732d9379aebf49986b1a"
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
