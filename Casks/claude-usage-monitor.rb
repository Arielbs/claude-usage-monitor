cask "claude-usage-monitor" do
  version "0.2.0"
  sha256 "b5582dd3a9d1465f84d6923802021d2d342214bb70f566d9c29c2c53637a321d"

  url "https://github.com/Arielbs/claude-usage-monitor/releases/download/v#{version}/Claude.Usage.Monitor_#{version}_aarch64.dmg"
  name "Claude Usage Monitor"
  desc "macOS menu bar app that displays your Claude AI usage limits"
  homepage "https://github.com/Arielbs/claude-usage-monitor"

  depends_on macos: ">= :monterey"

  app "Claude Usage Monitor.app"

  zap trash: [
    "~/.claude-usage-monitor-profile",
  ]
end
