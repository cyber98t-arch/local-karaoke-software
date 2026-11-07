[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

$operatingSystem = Get-CimInstance Win32_OperatingSystem
$computer = Get-CimInstance Win32_ComputerSystem
$processor = Get-CimInstance Win32_Processor | Select-Object -First 1
$adapters = @(Get-CimInstance Win32_VideoController | ForEach-Object {
    [ordered]@{
        name = $_.Name
        adapterCompatibility = $_.AdapterCompatibility
        driverVersion = $_.DriverVersion
        driverDate = if ($null -eq $_.DriverDate) { $null } else { $_.DriverDate.ToString("o") }
        pnpDeviceId = $_.PNPDeviceID
        status = $_.Status
        adapterRamBytes = if ($null -eq $_.AdapterRAM) { $null } else { [uint64]$_.AdapterRAM }
        videoProcessor = $_.VideoProcessor
    }
})

$report = [ordered]@{
    schemaVersion = 1
    collectedAtUtc = [DateTime]::UtcNow.ToString("o")
    operatingSystem = [ordered]@{
        caption = $operatingSystem.Caption
        version = $operatingSystem.Version
        buildNumber = $operatingSystem.BuildNumber
        architecture = $operatingSystem.OSArchitecture
    }
    computer = [ordered]@{
        manufacturer = $computer.Manufacturer
        model = $computer.Model
        totalPhysicalMemoryBytes = [uint64]$computer.TotalPhysicalMemory
    }
    processor = [ordered]@{
        name = $processor.Name
        logicalProcessors = $processor.NumberOfLogicalProcessors
    }
    videoAdapters = $adapters
}

$report | ConvertTo-Json -Depth 5
