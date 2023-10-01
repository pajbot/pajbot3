function Invoke-NativeCommand() {
    # A handy way to run a command, and automatically throw an error if the
    # exit code is non-zero.

    if ($args.Count -eq 0) {
        throw "Must supply some arguments."
    }

    $command = $args[0]
    $commandArgs = @()
    if ($args.Count -gt 1) {
        $commandArgs = $args[1..($args.Count - 1)]
    }

    & $command $commandArgs
    $result = $LASTEXITCODE

    if ($result -ne 0) {
        throw "$command $commandArgs exited with code $result."
    }
}

# This is only for PowerShell exceptions. PowerShell does not consider nonzero exit codes to be errors.
# The helper function above takes care of those.
$ErrorActionPreference = "Stop"

# If changing any of these commands, make the same changes to the workflows in .github/workflows!
Invoke-NativeCommand cargo fmt
Invoke-NativeCommand cargo check --all-targets --all-features
Invoke-NativeCommand cargo test
Invoke-NativeCommand cargo clippy --all-targets --all-features -- -D warnings

try {
    Set-Location web

    Invoke-NativeCommand npm install
    Invoke-NativeCommand npx --no-install prettier --write .
    Invoke-NativeCommand npm run build

    echo "All ok!"
}
catch {
    Write-Host "An error occurred: $_"
    exit 1
}
finally {
    # Restore the original working directory
    Set-Location ..
}
