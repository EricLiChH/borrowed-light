$ErrorActionPreference = "Stop"

$courseMdbookBin = if ($env:MDBOOK_BIN) { $env:MDBOOK_BIN } else { "mdbook" }
$courseRustlingsBin = if ($env:RUSTLINGS_BIN) { $env:RUSTLINGS_BIN } else { "rustlings" }

function Invoke-CourseCheck {
    param(
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    & $Command
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Invoke-CourseCheck { cargo fmt --all --check }
Invoke-CourseCheck { cargo clippy --workspace --all-targets -- -D warnings }
Invoke-CourseCheck { cargo test --workspace }
Invoke-CourseCheck { & $courseMdbookBin test book }
Invoke-CourseCheck { & $courseMdbookBin build book }

Push-Location exercises
try {
    Invoke-CourseCheck { & $courseRustlingsBin dev check }
}
finally {
    Pop-Location
}
