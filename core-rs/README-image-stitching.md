# core-rs Image Stitching

This is the initial Rust-only implementation of the shared image processing
and stitching API.

## Current support

- core-image: decode/encode PNG, JPEG, WebP and BMP through the image crate,
  normalize to RGBA, and enforce a default pixel budget.
- core-stitch: vertical, horizontal and grid layouts; pure-Rust overlap
  search for consecutive vertical scrolling screenshots; automatic fixed top/bottom
  bar detection and cropping; ambiguity penalties for repeated content; configurable
  overlap range for irregular scroll distances; JSON configuration and result metadata.
- Compose Multiplatform: image selection on Android and desktop, manual ordering,
  scroll/layout mode selection, indeterminate progress, output preview and export path.
- Android/JVM JNI: CoreRsNative.nativeStitch(inputPathsJson, outputPath,
  configJson). Inputs are filesystem paths. Android content:// URIs must first
  be copied to a readable temporary file by the platform layer.
- C FFI: core_rs_stitch_files; returned JSON memory must be released with the
  existing core_rs_string_free function.

## Configuration JSON

All fields are optional because the deserializer starts from
StitchConfig::default(). Example:

    {
      "mode": "vertical",
      "max_width": 4096,
      "max_height": 30000,
      "min_overlap_ratio": 0.05,
      "max_overlap_ratio": 0.95,
      "auto_detect_fixed_bars": true,
      "fixed_header_height": 0,
      "fixed_footer_height": 0,
      "min_confidence": 0.45,
      "blend": false,
      "output_format": "png",
      "columns": 2,
      "spacing": 12,
      "background": [255, 255, 255, 255],
      "max_output_pixels": 100000000
    }

Supported modes are scroll, panorama, vertical, horizontal, grid and auto.
In this first milestone, panorama mode returns a clear error because the
optional OpenCV backend is not wired in yet. Auto currently uses the
conservative scrolling screenshot matcher; it does not silently apply
panorama transforms.

## Build and test

From core-rs:

    cargo test -p core-image
    cargo test -p core-stitch
    cargo check -p core-rs-android
    cargo check -p core-rs-jvm
    cargo check -p core-rs-ffi

The workspace lockfile must be refreshed by Cargo after adding the new image
dependency. Run cargo check or cargo test without --locked once in a
network-enabled Rust environment, then commit the resulting Cargo.lock.

## Planned next steps

1. Add multi-scale and local-feature fallback matching for low-texture screenshots,
   plus optional manual fixed-bar crop overrides in the UI.
2. Add cancellable task handles and native incremental progress reporting.
3. Only after the core matcher and UI are stable, add the optional
   core-stitch-opencv crate for ORB/RANSAC panorama registration and seam blending.
