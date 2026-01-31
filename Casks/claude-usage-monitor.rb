cask "claude-usage-monitor" do
  version "0.1.0"
  sha256 "c682982427398dc03d0c93736c6fbb88106906b806733b35ce7ab332e1cb616a"

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
