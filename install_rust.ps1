# PowerShell script to install Rust
Write-Host "Installing Rust..." -ForegroundColor Green

# Download rustup-init.exe
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"

# Run installer with default options
Start-Process -FilePath ".\rustup-init.exe" -ArgumentList "-y" -Wait

# Remove installer
Remove-Item "rustup-init.exe"

# Refresh PATH
$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "User") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "Machine")

Write-Host "Rust installation complete!" -ForegroundColor Green
Write-Host "Please restart your terminal and run: cargo --version" -ForegroundColor Yellow
