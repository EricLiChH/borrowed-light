$ErrorActionPreference = "Stop"

$courseMdbookBin = if ($env:MDBOOK_BIN) { $env:MDBOOK_BIN } else { "mdbook" }
$courseRustlingsBin = if ($env:RUSTLINGS_BIN) { $env:RUSTLINGS_BIN } else { "rustlings" }
$coursePythonBin = if ($env:PYTHON_BIN) { $env:PYTHON_BIN } else { "python" }

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
Invoke-CourseCheck { cargo clippy --locked --workspace --all-targets -- -D warnings }
Invoke-CourseCheck { cargo test --locked --workspace }
Invoke-CourseCheck { & $coursePythonBin scripts/check_links.py }
Invoke-CourseCheck { & $courseMdbookBin test book }
Invoke-CourseCheck { & $courseMdbookBin build book }

Push-Location exercises
try {
    Invoke-CourseCheck { & $courseRustlingsBin dev check }
}
finally {
    Pop-Location
}
