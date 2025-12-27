# KhronosGroup conformance tests

Original tests: https://github.com/KhronosGroup/WebGL/
Path in original repo: /sdk/tests/conformance2/**

Checked out locally (DO NOT MODIFY!): /external/KhronosGroup-WebGL/sdk/tests/conformance2/**

These tests are part of the KhronosGroup WebGL conformance test suite, and our goal is to port them to run in our environment.

The original tests are in HTML with various tools and utilities loaded, which also makes it harder to debug.

Our goal is to port them to native built-in node.js test runner, using ES modules, and no dependencies (except our webgl2 library).