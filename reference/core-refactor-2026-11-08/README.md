# Core extraction verification

Current source changes after published 0.2.0. qsl-core owns schema/runtime contracts; lexer and facade preserve import paths by re-export. No runtime parser/generator dependency exists in core. Checked and unchecked APIs remain separate. Lexer/backend/runtime suites, generated contracts, examples and strict Clippy are recorded here. Fresh Criterion data and exact measured-source archives are under qsl-bench/strings/results. These results are not registry-release validation of a newly published package.

Source-manifest hashes record local working-copy bytes; checkout line-ending conversion can differ. The benchmark source ZIP files retain the exact measured inputs. A final cold generator lint cleanup leaves generated codecs byte-identical, as recorded in generated-codec-identity.json.
