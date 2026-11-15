---
title: "Hashes and compatibility"
description: "QSL documentation: Hashes and compatibility."
---

Analysis computes strict and light schema identities. Strict includes schema/field names; light omits names while preserving wire-affecting types, bounds, attributes and deterministic field order. xxHash is the default; xxHash128 is available through USE_HASH. Neither is authentication or a mathematical guarantee against all hash collisions.

Physical layout, CPU profile, inline policy and other options that do not change input/output bytes are excluded. Auto-wire v1 is included. Keep schema configuration with fixture bytes and generator version. A hash does not validate a received frame or make different revisions interchangeable.
