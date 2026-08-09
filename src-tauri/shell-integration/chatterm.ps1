# Chatterm shell integration for PowerShell and Windows PowerShell.

$script:ChattermEscape = [char]27
$script:ChattermCommandActive = $false
$script:ChattermOriginalPrompt = (Get-Command prompt -ErrorAction SilentlyContinue).ScriptBlock

function script:Write-ChattermMarker([string]$Payload) {
    [Console]::Write("$($script:ChattermEscape)]777;$Payload`a")
}

function global:prompt {
    $commandSucceeded = $?
    $nativeExitCode = $global:LASTEXITCODE
    $exitCode = if ($null -ne $nativeExitCode) { $nativeExitCode } elseif ($commandSucceeded) { 0 } else { 1 }

    if ($script:ChattermCommandActive) {
        Write-ChattermMarker "D;$exitCode"
        $script:ChattermCommandActive = $false
    }

    Write-ChattermMarker "A;$($executionContext.SessionState.Path.CurrentLocation.Path)"
    $promptText = if ($null -ne $script:ChattermOriginalPrompt) {
        & $script:ChattermOriginalPrompt
    } else {
        "PS $($executionContext.SessionState.Path.CurrentLocation)> "
    }
    Write-ChattermMarker "B"
    return $promptText
}

try {
    Import-Module PSReadLine -ErrorAction Stop
    Set-PSReadLineKeyHandler -Chord Enter -ScriptBlock {
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        Write-ChattermMarker "C;$line"
        $script:ChattermCommandActive = $true
        $global:LASTEXITCODE = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }
} catch {
    # PowerShell remains fully usable without enhanced command markers.
}
