git.exe log dev..origin/draft --oneline --reverse | ForEach-Object {
    $hash = ($_ -split ' ')[0]

    git.exe cherry-pick $hash -n

    $msgLines = git.exe log -1 --pretty=format:%B $hash -r | Out-String -Stream

    $commitArgs = @()
    foreach ($line in $msgLines) {
        $commitArgs += "-m"
        $commitArgs += $line
    }

    git.exe commit @commitArgs
}
