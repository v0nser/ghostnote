fn main() {
    #[cfg(target_os = "macos")]
    link_swift_runtime();

    tauri_build::build()
}

/// The `screencapturekit` crate bridges to ScreenCaptureKit through Swift, so
/// linking needs the Swift runtime and its compatibility shims. Its build
/// script only emits the Xcode toolchain path, which does not exist on a
/// machine with just the Command Line Tools installed — even though the
/// Command Line Tools ship the very same libraries at a different prefix.
///
/// Adding the fallback path here means a full Xcode install is not required to
/// build GhostNote.
#[cfg(target_os = "macos")]
fn link_swift_runtime() {
    const XCODE_SWIFT: &str =
        "/Library/Developer/CommandLineTools/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx";
    const CLT_SWIFT: &str = "/Library/Developer/CommandLineTools/usr/lib/swift/macosx";

    println!("cargo:rerun-if-changed=build.rs");

    // At *runtime* the Swift standard library is provided by the OS at
    // /usr/lib/swift (served from the dyld shared cache). Without this rpath,
    // dyld searches only the build directories baked in by the Swift bridge's
    // build script and aborts with a missing libswift_Concurrency.dylib.
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");

    if std::path::Path::new(XCODE_SWIFT).exists() {
        return;
    }

    if std::path::Path::new(CLT_SWIFT).exists() {
        println!("cargo:rustc-link-search=native={CLT_SWIFT}");
    } else {
        println!(
            "cargo:warning=Swift runtime libraries not found; install the Xcode Command Line Tools"
        );
    }
}
