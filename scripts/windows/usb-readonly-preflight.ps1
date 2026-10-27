#Requires -Version 5.1
[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)][string]$ExpectedFriendlyName,
  [Parameter(Mandatory=$true)][UInt64]$ExpectedSizeBytes,
  [string]$ExpectedSerial,
  [switch]$AllowVirtual
)
$ErrorActionPreference = 'Stop'
$matches = @(Get-Disk | Where-Object {
  $_.FriendlyName -eq $ExpectedFriendlyName -and
  [UInt64]$_.Size -eq $ExpectedSizeBytes -and
  ($AllowVirtual -or $_.BusType -eq 'USB') -and
  -not $_.IsBoot -and -not $_.IsSystem -and -not $_.IsOffline
})
if ($ExpectedSerial) {
  $matches = @($matches | Where-Object { $_.SerialNumber -and $_.SerialNumber.Trim() -eq $ExpectedSerial.Trim() })
}
if ($matches.Count -ne 1) { throw "FAIL CLOSED: expected exactly one non-system matching disk; found $($matches.Count). No writes performed." }
$disk = $matches[0]
if ($disk.IsReadOnly) { Write-Host 'NOTE: target disk is read-only' }
Write-Host "PASS: read-only identity check for disk $($disk.Number), $($disk.FriendlyName), $($disk.Size) bytes, bus $($disk.BusType). No writes performed."
