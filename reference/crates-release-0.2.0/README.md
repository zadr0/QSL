# QSL 0.2.0 release verification

Source checks, generated contracts, examples, strict Clippy, Rustdoc, Lean and website/SEO validation are retained here. All three packages were published to crates.io. Their cached registry archives match the registry SHA-256 checksums in `registry-packages.json`.

`registry-consumer/` uses exact registry dependencies without path overrides or patches. Six tests passed with default features and six with `full`; the logs retain the actual runs. Package VCS metadata identifies the source commit. The release tag adds publication evidence and documentation maintenance without changing library source; every `source-manifest.json` hash was rechecked.

The noisy November 7 timing pass is not accepted as strict performance evidence. Lean checks the documented bitmap arithmetic, not the entire generated Rust memory model. GitHub Pages deployment and IndexNow receipt are checked separately after publication; submission does not guarantee indexing.
