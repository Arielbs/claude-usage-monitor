cask "claude-usage-monitor" do
  version "0.1.0"
  sha256 "e95f8ccd4657fb8cf4167e10d943e36a894295122623f043f8ee92bd64151d35"

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
