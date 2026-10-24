class Noot < Formula
  desc "Honks on every keypress"
  homepage "https://github.com/bachrc/nootnoot"
  head "https://github.com/bachrc/nootnoot.git", branch: "main"

  depends_on :linux
  depends_on "rust" => :build
  depends_on "pkgconf" => :build
  depends_on "alsa-lib"

  def fetch
    system "cargo", "fetch", *std_cargo_fetch_args
  end

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      Keyboard access is required once:

        sudo usermod -aG input "$USER"

      Log out and back in, then launch "Noot mode" from the application menu.

      The systemd unit lives outside the Homebrew prefix:
        ~/.config/systemd/user/nootd.service
      Remove it by hand after `brew uninstall`, then run
        systemctl --user daemon-reload
    EOS
  end

  test do
    assert_match(/\A\d+\.\d+\.\d+\z/, shell_output("#{bin}/noot-settings --version").strip)
  end
end
