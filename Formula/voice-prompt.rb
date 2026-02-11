class VoicePrompt < Formula
  desc "Voice-to-text with local Whisper transcription and AI refinement"
  homepage "https://github.com/tr4m0ryp/voice-prompt"
  url "https://github.com/tr4m0ryp/voice-prompt/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "cmake" => :build
  depends_on "pkg-config" => :build
  depends_on "gtk4"
  depends_on "libadwaita"

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  def caveats
    <<~EOS
      Voice Prompt requires Accessibility permissions for global hotkeys.
      Go to: System Settings > Privacy & Security > Input Monitoring
      Add 'voice-prompt' (or your terminal) to the list.

      To start on login:
        mkdir -p ~/Library/LaunchAgents
        cp #{prefix}/com.voice-prompt.agent.plist ~/Library/LaunchAgents/
        launchctl load ~/Library/LaunchAgents/com.voice-prompt.agent.plist
    EOS
  end

  test do
    assert_match "voice-prompt", shell_output("#{bin}/voice-prompt --help 2>&1", 0)
  end
end
