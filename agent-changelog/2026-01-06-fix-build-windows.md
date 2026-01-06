Summary
-------
Fixed a syntax error in scripts\build-windows.ps1 that caused build-windows.bat to fail.

Details
-------
- Problem: The here-string closing token "@ was indented, which is not allowed in PowerShell and caused a parser error and apparent mismatched braces.
- Change: Removed leading spaces before the here-string terminator so it appears at column 1.
- Validation: Ran build-windows.bat; the PowerShell script executed and the full build completed with exit code 0. Noted minor warnings from npm and inability to use System.Web.HttpUtility in the HTML rendering step on this environment, but these do not cause the build to fail.

Files modified
--------------
- scripts\build-windows.ps1: adjusted here-string terminator line (removed leading spaces).

Notes
-----
This is a minimal surgical change to correct PowerShell here-string syntax.
