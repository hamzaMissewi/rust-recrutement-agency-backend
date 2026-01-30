# PowerShell script to install PostgreSQL using Chocolatey
Write-Host "Installing PostgreSQL..." -ForegroundColor Green

# Check if Chocolatey is installed
if (!(Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Chocolatey..." -ForegroundColor Yellow
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
}

# Install PostgreSQL
choco install postgresql --params '/Password:password' -y

Write-Host "PostgreSQL installation complete!" -ForegroundColor Green
Write-Host "Please restart your terminal and run: psql --version" -ForegroundColor Yellow
