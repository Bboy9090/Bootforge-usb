$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'usb-readonly-preflight.ps1'
$failed = $false
try { & $script -ExpectedFriendlyName 'INFINITEBOOT_NONEXISTENT_TEST_DEVICE' -ExpectedSizeBytes 62914560000 | Out-Null } catch { $failed = $true }
if (-not $failed) { throw 'FAIL: nonexistent disk must be rejected' }
$failed = $false
try { & $script -ExpectedFriendlyName 'INFINITEBOOT_NONEXISTENT_TEST_DEVICE' -ExpectedSizeBytes 1 -ExpectedSerial 'INVALID' | Out-Null } catch { $failed = $true }
if (-not $failed) { throw 'FAIL: nonexistent disk with serial must be rejected' }
Write-Host 'PASS: Windows native fail-closed negative tests; no disk writes'
