Rolling builds of the `nightly-remote` branch for macOS Apple Silicon and Windows x64: everything on `main` plus [web remote control](https://github.com/yicheng47/runner/issues/836) while it is in development. They are identified by commit and UTC build stamp and display as **Remote Nightly (<commit>)**. Remote nightly installs update from this release only; the regular nightly and production builds keep their own feeds. Every channel uses the same app and data, so settings, chats and missions carry over when you switch. Install a nightly or stable build by hand to switch back.

## macOS

Download the newest **Runner-Remote-Nightly-…-arm64.dmg** below and drag **Runner.app** into Applications, replacing Runner. The app checks this release's signed Sparkle appcast for updates. The DMG is signed and notarized.

## Windows

Download the newest **Runner-Setup-remote-nightly…-x64.exe** installer below and run it over your current Runner. It installs for your Windows account without administrator access. Runner checks this release for newer remote nightlies at startup and every six hours, verifies their minisign signatures, and installs them from **Settings → Updates → Update**, the same way as the regular nightly.
