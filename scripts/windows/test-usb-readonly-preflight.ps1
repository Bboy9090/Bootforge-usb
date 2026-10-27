$ErrorActionPreference = 'Stop'
$script = Join-Path $PSScriptRoot 'usb-readonly-preflight.ps1'
function Assert-Rejected([string]$name, [UInt64]$size, [string]$serial) {
  $rejected = $false
  try { & $script -ExpectedFriendlyName $name -ExpectedSizeBytes $size -ExpectedSerial $serial | Out-Null } catch { $rejected = $true }
  if (-not $rejected) { throw "FAIL: invalid disk identity was accepted: $name" }
}
Assert-Rejected 'INFINITEBOOT_NONEXISTENT_TEST_DEVICE' 62914560000 ''
Assert-Rejected 'INFINITEBOOT_NONEXISTENT_TEST_DEVICE' 1 'INVALID'
# Use a real, non-USB, non-system virtual disk exposed by the CI host only if one exists.
# The default preflight must reject it because the USB bus constraint is mandatory.
$virtual = @(Get-Disk | Where-Object { $_.BusType -ne 'USB' -and -not $_.IsBoot -and -not $_.IsSystem -and -not $_.IsOffline } | Select-Object -First 1)
if ($virtual.Count -gt 0) {
  Assert-Rejected $virtual[0].FriendlyName ([UInt64]$virtual[0].Size) $virtual[0].SerialNumber
  Write-Host 'PASS: non-USB disk rejected even with matching name, size and serial'
} else { Write-Host 'SKIP: no eligible non-USB non-system disk for bus-negative test' }
Write-Host 'PASS: Windows native fail-closed negative tests; no disk writes'
