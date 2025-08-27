param(
    [Parameter(Mandatory=$true)]
    [string]$SourceBranch,

    [Parameter(Mandatory=$true)]
    [string]$TargetBranch
)

git.exe log "$SourceBranch..$TargetBranch" --oneline --reverse | ForEach-Object {
    $hash = ($_ -split ' ')[0]

    git.exe cherry-pick $hash -n

    $msgLines = git.exe log -1 --pretty=format:%B $hash | Out-String -Stream | Where-Object { $_.Trim() -ne "" }

    $commitArgs = @()
    foreach ($line in $msgLines) {
        $commitArgs += "-m"
        $commitArgs += $line
    }

    git.exe commit @commitArgs
}
