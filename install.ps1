# OpenCoWork Rust — Windows Installer
#
# Automates full setup: Rust, llama.cpp, OpenCoWork, models
# Run in PowerShell as Administrator:
#   irm https://raw.githubusercontent.com/multidimensionalinteractive/opencowork-rust/main/install.ps1 | iex
#
# Or download and run:
#   .\install.ps1

param(
    [string]$InstallDir = "$env:USERPROFILE\opencowork",
    [switch]$SkipRust,
    [switch]$SkipLlama,
    [switch]$SkipModels,
    [switch]$Cuda,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$VERSION = "0.1.0"
$REPO = "multidimensionalinteractive/opencowork-rust"
$LLAMA_CPP_REPO = "ggml-org/llama.cpp"

# Colors for output
function Write-Color($Text, $Color = "White") {
    Write-Host $Text -ForegroundColor $Color
}

function Write-Header($Text) {
    Write-Host ""
    Write-Host "  ╔══════════════════════════════════════════════╗" -ForegroundColor DarkCyan
    Write-Host "  ║  $Text" -ForegroundColor Cyan
    Write-Host "  ╔══════════════════════════════════════════════╝" -ForegroundColor DarkCyan
    Write-Host ""
}

function Write-Step($Text) {
    Write-Host "  → " -ForegroundColor DarkGray -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Write-Done($Text) {
    Write-Host "  ✓ " -ForegroundColor Green -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Write-Skip($Text) {
    Write-Host "  ⊘ " -ForegroundColor Yellow -NoNewline
    Write-Host $Text -ForegroundColor Gray
}

function Write-Error($Text) {
    Write-Host "  ✗ " -ForegroundColor Red -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Test-Command($Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Test-NvidiaGpu() {
    try {
        $gpu = & nvidia-smi --query-gpu=name --format=csv,noheader 2>$null
        return [bool]$gpu
    } catch {
        return $false
    }
}

# ─── ASCII Banner ───
function Show-Banner {
    Write-Host ""
    Write-Host "  ██████╗ ██████╗ ███████╗███╗   ██╗ ██████╗ ██████╗ ██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗" -ForegroundColor Magenta
    Write-Host " ██╔═══██╗██╔══██╗██╔════╝████╗  ██║██╔═══██╗██╔══██╗██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝" -ForegroundColor Magenta
    Write-Host " ██║   ██║██████╔╝█████╗  ██╔██╗ ██║██║   ██║██████╔╝██║ █╗ ██║██║   ██║██████╔╝█████╔╝ " -ForegroundColor Magenta
    Write-Host " ██║   ██║██╔═══╝ ██╔══╝  ██║╚██╗██║██║   ██║██╔══██╗██║███╗██║██║   ██║██╔══██╗██╔═██╗ " -ForegroundColor Magenta
    Write-Host " ╚██████╔╝██║     ███████╗██║ ╚████║╚██████╔╝██║  ██║╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗" -ForegroundColor Magenta
    Write-Host "  ╚═════╝ ╚═╝     ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝ ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝" -ForegroundColor Magenta
    Write-Host ""
    Write-Host "    ╔═══════════════════════════════════════════════════════╗" -ForegroundColor DarkCyan
    Write-Host "    ║  🦀 RUST-POWERED  •  ⚡ BLAZING FAST  •  🔒 PRIVATE  ║" -ForegroundColor Cyan
    Write-Host "    ║  Windows Installer v$VERSION                            ║" -ForegroundColor Cyan
    Write-Host "    ╚═══════════════════════════════════════════════════════╝" -ForegroundColor DarkCyan
    Write-Host ""
}

# ─── Help ───
function Show-Help {
    Show-Banner
    Write-Color "  OpenCoWork Rust — Windows Installer" "White"
    Write-Host ""
    Write-Color "  USAGE:" "Yellow"
    Write-Host "    .\install.ps1 [options]"
    Write-Host ""
    Write-Color "  OPTIONS:" "Yellow"
    Write-Host "    -InstallDir <path>   Installation directory (default: ~\opencowork)"
    Write-Host "    -SkipRust            Skip Rust installation"
    Write-Host "    -SkipLlama           Skip llama.cpp installation"
    Write-Host "    -SkipModels          Skip model downloads"
    Write-Host "    -Cuda                Install CUDA-enabled llama.cpp (requires NVIDIA GPU)"
    Write-Host "    -Help                Show this help"
    Write-Host ""
    Write-Color "  EXAMPLES:" "Yellow"
    Write-Host "    .\install.ps1                              # Full install"
    Write-Host "    .\install.ps1 -Cuda                        # With GPU acceleration"
    Write-Host "    .\install.ps1 -InstallDir D:\ai\opencowork  # Custom path"
    Write-Host "    .\install.ps1 -SkipModels                   # Don't download models"
    Write-Host ""
    exit 0
}

if ($Help) { Show-Help }

# ─── Pre-flight Checks ───
Show-Banner

Write-Color "  Checking system..." "DarkGray"
Write-Host ""

# Check Windows version
$winVer = [System.Environment]::OSVersion.Version
if ($winVer.Major -lt 10) {
    Write-Error "Windows 10 or later required"
    exit 1
}

# Check architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
if ($arch -eq "x86") {
    Write-Error "64-bit Windows required"
    exit 1
}

# Check disk space (need at least 10GB)
$drive = (Get-Item $InstallDir -ErrorAction SilentlyContinue).PSDrive.Name
if (-not $drive) { $drive = $env:SystemDrive.TrimEnd(":") }
$freeGB = [math]::Round((Get-PSDrive $drive).Free / 1GB, 1)
if ($freeGB -lt 10) {
    Write-Error "Need at least 10GB free space (have ${freeGB}GB)"
    exit 1
}

Write-Done "Windows 10+ ($arch), ${freeGB}GB free"

# Check for NVIDIA GPU
$hasNvidia = Test-NvidiaGpu
if ($hasNvidia) {
    $gpuName = & nvidia-smi --query-gpu=name --format=csv,noheader 2>$null | Select-Object -First 1
    $vramMiB = & nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>$null | Select-Object -First 1
    $vramGB = [math]::Round([int]$vramMiB / 1024, 1)
    Write-Done "NVIDIA GPU detected: $gpuName (${vramGB}GB VRAM)"
    if (-not $Cuda) {
        Write-Skip "Tip: Add -Cuda flag for GPU acceleration"
    }
} else {
    Write-Skip "No NVIDIA GPU detected (CPU-only inference)"
}

# ─── Create directories ───
Write-Header "SETUP DIRECTORIES"

$dirs = @(
    $InstallDir,
    "$InstallDir\bin",
    "$InstallDir\models",
    "$InstallDir\config",
    "$InstallDir\logs"
)

foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Done "Created $dir"
    } else {
        Write-Skip "Exists: $dir"
    }
}

# ─── Install Rust ───
Write-Header "RUST TOOLCHAIN"

if ($SkipRust -or (Test-Command "rustc")) {
    if (Test-Command "rustc") {
        $rustVer = & rustc --version
        Write-Skip "Rust already installed: $rustVer"
    } else {
        Write-Skip "Skipping Rust (-SkipRust)"
    }
} else {
    Write-Step "Downloading rustup-init.exe..."
    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

    Write-Step "Installing Rust (this takes a minute)..."
    & $rustupPath -y --default-toolchain stable 2>$null

    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    if (Test-Command "rustc") {
        $rustVer = & rustc --version
        Write-Done "Rust installed: $rustVer"
    } else {
        Write-Error "Rust installation failed — install manually from https://rustup.rs"
        exit 1
    }
}

# ─── Build OpenCoWork ───
Write-Header "BUILD OPENCOWORK"

$buildDir = "$InstallDir\source"
if (-not (Test-Path $buildDir)) {
    Write-Step "Cloning repository..."
    & git clone "https://github.com/$REPO.git" $buildDir 2>$null
    Write-Done "Cloned to $buildDir"
} else {
    Write-Step "Updating repository..."
    Push-Location $buildDir
    & git pull 2>$null
    Pop-Location
    Write-Done "Updated $buildDir"
}

Write-Step "Building server (release mode)..."
Push-Location $buildDir
$buildStart = Get-Date
& cargo build --release -p opencowork-server 2>$null
$buildTime = [math]::Round(((Get-Date) - $buildStart).TotalSeconds, 1)

if ($LASTEXITCODE -eq 0) {
    Copy-Item "$buildDir\target\release\opencowork-server.exe" "$InstallDir\bin\" -Force
    Write-Done "Server built in ${buildTime}s → $InstallDir\bin\opencowork-server.exe"
} else {
    Write-Error "Server build failed"
}

Write-Step "Building router..."
& cargo build --release -p opencowork-router 2>$null
if ($LASTEXITCODE -eq 0) {
    Copy-Item "$buildDir\target\release\opencowork-router.exe" "$InstallDir\bin\" -Force
    Write-Done "Router built → $InstallDir\bin\opencowork-router.exe"
} else {
    Write-Error "Router build failed"
}
Pop-Location

# ─── Install llama.cpp ───
Write-Header "LLAMA.CPP (LOCAL INFERENCE)"

if ($SkipLlama) {
    Write-Skip "Skipping llama.cpp (-SkipLlama)"
} elseif (Test-Command "llama-server") {
    Write-Skip "llama.cpp already in PATH"
} else {
    $llamaDir = "$InstallDir\llama.cpp"

    if (-not (Test-Path $llamaDir)) {
        Write-Step "Cloning llama.cpp..."
        & git clone "https://github.com/$LLAMA_CPP_REPO.git" $llamaDir 2>$null
    } else {
        Write-Step "Updating llama.cpp..."
        Push-Location $llamaDir
        & git pull 2>$null
        Pop-Location
    }

    if ($Cuda -and $hasNvidia) {
        Write-Step "Building llama.cpp with CUDA acceleration..."
        $cudaPath = $env:CUDA_PATH
        if (-not $cudaPath) {
            $cudaPath = "C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA"
            if (Test-Path $cudaPath) {
                $cudaVersions = Get-ChildItem $cudaPath -Directory | Sort-Object Name -Descending | Select-Object -First 1
                $cudaPath = $cudaVersions.FullName
            }
        }

        Push-Location $llamaDir
        & cmake -B build -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES="75;80;86;89;90" 2>$null
        & cmake --build build --config Release -j $env:NUMBER_OF_PROCESSORS 2>$null
        Pop-Location

        if ($LASTEXITCODE -eq 0) {
            Write-Done "llama.cpp built with CUDA → $llamaDir\build\bin\"
        } else {
            Write-Error "CUDA build failed, trying CPU build..."
            Push-Location $llamaDir
            & cmake -B build 2>$null
            & cmake --build build --config Release -j $env:NUMBER_OF_PROCESSORS 2>$null
            Pop-Location
        }
    } else {
        Write-Step "Building llama.cpp (CPU mode)..."
        Push-Location $llamaDir
        & cmake -B build 2>$null
        & cmake --build build --config Release -j $env:NUMBER_OF_PROCESSORS 2>$null
        Pop-Location
    }

    if ($LASTEXITCODE -eq 0) {
        # Add to PATH
        $binPath = "$llamaDir\build\bin\Release"
        if (-not (Test-Path $binPath)) { $binPath = "$llamaDir\build\bin" }

        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$binPath*") {
            [Environment]::SetEnvironmentVariable("Path", "$currentPath;$binPath", "User")
            $env:Path += ";$binPath"
        }

        Write-Done "llama.cpp installed → $binPath"
    } else {
        Write-Error "llama.cpp build failed"
        Write-Skip "Install manually: https://github.com/ggml-org/llama.cpp/releases"
    }
}

# ─── Download Models ───
Write-Header "MODELS"

if ($SkipModels) {
    Write-Skip "Skipping model downloads (-SkipModels)"
} else {
    $modelsDir = "$InstallDir\models"

    # Detect available VRAM for model selection
    $vramGB = 0
    if ($hasNvidia) {
        $vramMiB = & nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>$null | Select-Object -First 1
        $vramGB = [math]::Round([int]$vramMiB / 1024, 1)
    }

    function Download-Model($Url, $Filename, $Description) {
        $outPath = "$modelsDir\$Filename"
        if (Test-Path $outPath) {
            Write-Skip "Already downloaded: $Filename"
            return
        }
        Write-Step "Downloading $Description..."
        Write-Step "  → $Filename"
        try {
            Invoke-WebRequest -Uri $Url -OutFile "$outPath.tmp" -Resume -ErrorAction Stop
            Move-Item "$outPath.tmp" $outPath -Force
            $sizeMB = [math]::Round((Get-Item $outPath).Length / 1MB, 0)
            Write-Done "$Description (${sizeMB}MB)"
        } catch {
            Write-Error "Download failed: $_"
            Remove-Item "$outPath.tmp" -ErrorAction SilentlyContinue
        }
    }

    # Always download a small model
    Download-Model `
        "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q4_K_M.gguf" `
        "Qwen2.5-7B-Instruct-Q4_K_M.gguf" `
        "Qwen 2.5 7B (default, works on any GPU with 5GB+ VRAM)"

    # Download based on VRAM
    if ($vramGB -ge 16) {
        Download-Model `
            "https://huggingface.co/bartowski/Qwen2.5-32B-Instruct-abliterated-GGUF/resolve/main/Qwen2.5-32B-Instruct-abliterated-Q4_K_M.gguf" `
            "Qwen2.5-32B-abliterated-Q4_K_M.gguf" `
            "Qwen 2.5 32B Uncensored (needs ~20GB VRAM)"
    }

    if ($vramGB -ge 18) {
        Download-Model `
            "https://huggingface.co/bartowski/gemma-4-27b-it-abliterated-GGUF/resolve/main/gemma-4-27b-it-abliterated-Q4_K_M.gguf" `
            "gemma-4-27b-abliterated-Q4_K_M.gguf" `
            "Gemma 4 27B Abliterated (needs ~18GB VRAM)"
    }

    if ($vramGB -ge 44) {
        Download-Model `
            "https://huggingface.co/NousResearch/Hermes-3-Llama-3.1-70B-GGUF/resolve/main/Hermes-3-Llama-3.1-70B.Q4_K_M.gguf" `
            "Hermes-3-70B-Q4_K_M.gguf" `
            "Hermes 3 Llama 70B Uncensored (needs ~40GB VRAM)"
    }
}

# ─── Create Config ───
Write-Header "CONFIGURATION"

$serverConfig = @"
# OpenCoWork Server Config
# Generated by Windows installer v$VERSION

host = "127.0.0.1"
port = 9876

# Approval mode: auto, manual, timeout
approval_mode = "timeout"
approval_timeout_secs = 30

# Authorized workspace roots (add your project dirs)
# workspaces = ["C:\Users\$env:USERNAME\projects"]

# CORS (empty = allow all for local use)
cors_origins = []
"@

$configPath = "$InstallDir\config\server.toml"
if (-not (Test-Path $configPath)) {
    $serverConfig | Out-File -FilePath $configPath -Encoding UTF8
    Write-Done "Server config → $configPath"
} else {
    Write-Skip "Config exists: $configPath"
}

$routerConfig = @"
# OpenCoWork Router Config

[[telegram]]
id = "main"
# token = "YOUR_BOT_TOKEN_HERE"

[router]
opencode_url = "http://localhost:9876"
dedup_window_secs = 30
"@

$routerConfigPath = "$InstallDir\config\router.toml"
if (-not (Test-Path $routerConfigPath)) {
    $routerConfig | Out-File -FilePath $routerConfigPath -Encoding UTF8
    Write-Done "Router config → $routerConfigPath"
} else {
    Write-Skip "Config exists: $routerConfigPath"
}

# ─── Create Launchers ───
Write-Header "SHORTCUTS"

# Start server batch file
$startServer = @"
@echo off
echo.
echo   🦀 Starting OpenCoWork Server...
echo.
cd /d "$InstallDir"
"$InstallDir\bin\opencowork-server.exe" --workspace "%cd%" --config "$InstallDir\config\server.toml"
pause
"@
$startServerPath = "$InstallDir\Start Server.bat"
$startServer | Out-File -FilePath $startServerPath -Encoding ASCII
Write-Done "Start Server → $startServerPath"

# Start llama server batch file
$startLlama = @"
@echo off
echo.
echo   🦙 Starting llama.cpp server...
echo.

set MODEL=%1
if "%MODEL%"=="" set MODEL="$InstallDir\models\Qwen2.5-7B-Instruct-Q4_K_M.gguf"

echo   Model: %MODEL%
echo   Port:  8080
echo.

"$InstallDir\llama.cpp\build\bin\Release\llama-server.exe" -m "%MODEL%" --host 127.0.0.1 --port 8080 -ngl 99 -c 32768 --chat-template chatml
pause
"@
$startLlamaPath = "$InstallDir\Start llama-server.bat"
$startLlama | Out-File -FilePath $startLlamaPath -Encoding ASCII
Write-Done "Start llama-server → $startLlamaPath"

# List models batch file
$listModels = @"
@echo off
echo.
echo   Available models in $InstallDir\models\
echo.
dir /b "$InstallDir\models\*.gguf" 2>nul
if errorlevel 1 echo   No models downloaded yet.
echo.
echo   Usage: "Start llama-server.bat" path\to\model.gguf
echo.
pause
"@
$listModelsPath = "$InstallDir\List Models.bat"
$listModels | Out-File -FilePath $listModelsPath -Encoding ASCII
Write-Done "List Models → $listModelsPath"

# Open frontend batch file
$startFrontend = @"
@echo off
echo.
echo   ⚡ Starting OpenCoWork Frontend...
echo.
cd /d "$InstallDir\source\apps\frontend"
bun install
bun dev
"@
$startFrontendPath = "$InstallDir\Start Frontend.bat"
$startFrontend | Out-File -FilePath $startFrontendPath -Encoding ASCII
Write-Done "Start Frontend → $startFrontendPath"

# Create desktop shortcut
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\OpenCoWork Server.lnk")
$Shortcut.TargetPath = $startServerPath
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.IconLocation = "shell32.dll,44"
$Shortcut.Description = "Start OpenCoWork Rust Server"
$Shortcut.Save()
Write-Done "Desktop shortcut created"

# ─── Add to PATH ───
Write-Header "PATH"

$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$InstallDir\bin*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$InstallDir\bin", "User")
    Write-Done "Added $InstallDir\bin to PATH"
} else {
    Write-Skip "Already in PATH: $InstallDir\bin"
}

# ─── Summary ───
Write-Header "DONE!"

Write-Host "  ┌──────────────────────────────────────────────────────┐" -ForegroundColor DarkCyan
Write-Host "  │  OpenCoWork Rust v$VERSION installed!                   │" -ForegroundColor Green
Write-Host "  │                                                      │" -ForegroundColor DarkCyan
Write-Host "  │  Install dir:  $InstallDir" -ForegroundColor White
Write-Host "  │  Binaries:     $InstallDir\bin" -ForegroundColor White
Write-Host "  │  Models:       $InstallDir\models" -ForegroundColor White
Write-Host "  │  Config:       $InstallDir\config" -ForegroundColor White
Write-Host "  │                                                      │" -ForegroundColor DarkCyan
Write-Host "  │  Quick start:                                        │" -ForegroundColor Yellow
Write-Host "  │    1. Double-click 'OpenCoWork Server' on Desktop    │" -ForegroundColor White
Write-Host "  │    2. Run 'Start llama-server.bat' for local LLM    │" -ForegroundColor White
Write-Host "  │    3. Open http://localhost:3000 in browser          │" -ForegroundColor White
Write-Host "  │                                                      │" -ForegroundColor DarkCyan
Write-Host "  │  Docs: github.com/$REPO     │" -ForegroundColor Cyan
Write-Host "  └──────────────────────────────────────────────────────┘" -ForegroundColor DarkCyan
Write-Host ""
