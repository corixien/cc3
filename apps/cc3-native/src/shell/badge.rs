//! Launcher badge (KDE Plasma and Unity-compatible docks) through the LauncherEntry D-Bus signal.
pub fn set(count: i64) {
    let props = format!("{{'count': <int64 {count}>, 'count-visible': <{}>}}", count > 0);
    let _ = std::process::Command::new("gdbus")
        .args([
            "emit",
            "--session",
            "--object-path",
            "/com/canonical/unity/launcherentry/1",
            "--signal",
            "com.canonical.Unity.LauncherEntry.Update",
            "application://CC3.desktop",
            &props,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
