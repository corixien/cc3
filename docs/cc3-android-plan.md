# CC3 Android plan

Goal: an Android app "CC3" built from `apps/mobile` (Expo / React Native, upstream T3 Code mobile)
that pairs with a CC3 desktop server and mirrors the CC3 branding. No new code base: the app is
already in the repo, so the work is configuration, branding and a local build chain.

## Findings (2026-11-22)

- `apps/mobile` is complete (Expo, `app.config.ts`, `eas.json`, plugins, native modules). No
  `android/` folder is committed; `expo prebuild` generates it (`pnpm android*` scripts).
- `app.config.ts` already names the app "CC3" / "CC3 Dev" / "CC3 Preview", but identifiers are still
  upstream: package `com.t3tools.t3code[.dev|.preview]`, schemes `t3code[-dev|-preview]`,
  Clerk relying party `clerk.t3.codes`.
- Upstream cloud pieces (Clerk login, T3 Connect relay, app.t3.codes deep links, EAS, Axiom
  telemetry) are driven by env vars (`T3CODE_RELAY_URL`, Clerk keys, OTLP). Unset = disabled.
- Machine: Android SDK in `~/Android/Sdk` (platforms 35/36/37, build-tools 35/36, NDK 29,
  emulator, AVD `Pixel_API35`), `adb`, `node_modules` installed. Java is OpenJDK 25 only; Gradle
  and React Native need JDK 17 or 21. No device or emulator is running.

## Steps

1. **Toolchain**: install JDK 21 (`dnf install java-21-openjdk-devel`), set `JAVA_HOME` and
   `ANDROID_HOME=~/Android/Sdk` for the build shell only; confirm `./gradlew` works after prebuild.
2. **Identity** (small edits in `app.config.ts`, keep iOS keys so upstream merges stay cheap):
   own package id `dev.corixien.cc3` (+ `.dev`, `.preview`), scheme `cc3`; drop the
   `app.t3.codes` intent filter and `assetlinks` expectations (and adapt `app.config.test.ts`).
3. **Branding**: replace Android assets (`android-icon-foreground/mark`, splash, notification
   icon, adaptive background color) with the CC3 logo; check the user-facing strings still saying
   "T3 Code" in `src/` and translations.
4. **Disable upstream cloud**: no `.env` (no Clerk/relay/OTLP); remove or hide the Clerk sign-in
   and T3 Connect screens so pairing is only "scan/enter CC3 server URL + pairing token".
   Verify the build with these features off (`features/cloud`, `connection`).
5. **Connection**: CC3 desktop server must be reachable from the phone: LAN or Tailscale host,
   pairing URL from `node apps/server/src/bin.ts pair` / desktop settings. Check that the server
   binds beyond 127.0.0.1 only when the user enables remote access, and that cleartext HTTP is
   allowed for LAN hosts on Android (network security config / `usesCleartextTraffic`).
6. **Build**: `APP_VARIANT=development pnpm android:dev` on the AVD first (debug, Metro), then a
   release APK (`gradlew assembleRelease` with a local keystore, no EAS) installable via `adb`.
7. **Feature parity with CC3 desktop** (later): wrapper/TUI threads, split view are desktop-only;
   decide what the phone shows (chat list, composer, approvals, terminal view).

## Open questions

- Package id / signing keystore location (proposal: `dev.corixien.cc3`, keystore outside repo).
- Release channel: sideload APK only, or also an update check against GitHub releases of `corixien/cc3`.
- Keep Clerk/T3 Connect code in the tree (merge-friendly, disabled) vs. remove it.
