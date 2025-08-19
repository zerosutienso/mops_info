Write-Host "MongoDB Setup Check" -ForegroundColor Green
Write-Host "==================" -ForegroundColor Green
Write-Host ""

Write-Host "1. Checking if MongoDB is installed..." -ForegroundColor Yellow
$mongodPath = Get-Command mongod -ErrorAction SilentlyContinue
if ($mongodPath) {
    Write-Host "[OK] MongoDB is installed" -ForegroundColor Green
    mongod --version | Select-String "db version"
} else {
    Write-Host "[ERROR] MongoDB is not installed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install MongoDB from:" -ForegroundColor Yellow
    Write-Host "https://www.mongodb.com/try/download/community" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Installation steps:" -ForegroundColor Yellow
    Write-Host "1. Download MongoDB Community Server for Windows"
    Write-Host "2. Run installer and choose 'Complete' installation"
    Write-Host "3. Check 'Install MongoDB as a Service'"
    Write-Host "4. Check 'Install MongoDB Compass'"
    Write-Host ""
    $choice = Read-Host "Open download page? (y/n)"
    if ($choice -eq "y" -or $choice -eq "Y") {
        Start-Process "https://www.mongodb.com/try/download/community"
    }
    exit 1
}

Write-Host ""
Write-Host "2. Checking MongoDB service..." -ForegroundColor Yellow
$service = Get-Service -Name "MongoDB" -ErrorAction SilentlyContinue
if ($service) {
    Write-Host "[OK] MongoDB service found - Status: $($service.Status)" -ForegroundColor Green
    if ($service.Status -ne "Running") {
        Write-Host "Attempting to start MongoDB service..." -ForegroundColor Yellow
        try {
            Start-Service -Name "MongoDB"
            Write-Host "[OK] MongoDB service started" -ForegroundColor Green
        } catch {
            Write-Host "[WARNING] Could not start MongoDB service (may need admin privileges)" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "[ERROR] MongoDB service not found" -ForegroundColor Red
    Write-Host "Please ensure 'Install MongoDB as a Service' was checked during installation"
}

Write-Host ""
Write-Host "3. Testing MongoDB connection..." -ForegroundColor Yellow
$mongoshPath = Get-Command mongosh -ErrorAction SilentlyContinue
$mongoPath = Get-Command mongo -ErrorAction SilentlyContinue

if ($mongoshPath) {
    try {
        $result = mongosh --eval "db.runCommand({ping: 1})" --quiet 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] MongoDB connection successful" -ForegroundColor Green
        } else {
            Write-Host "[ERROR] MongoDB connection failed" -ForegroundColor Red
        }
    } catch {
        Write-Host "[ERROR] MongoDB connection test failed" -ForegroundColor Red
    }
} elseif ($mongoPath) {
    try {
        $result = mongo --eval "db.runCommand({ping: 1})" --quiet 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] MongoDB connection successful (legacy mongo)" -ForegroundColor Green
        } else {
            Write-Host "[ERROR] MongoDB connection failed" -ForegroundColor Red
        }
    } catch {
        Write-Host "[ERROR] MongoDB connection test failed" -ForegroundColor Red
    }
} else {
    Write-Host "[WARNING] MongoDB client tools not found" -ForegroundColor Yellow
    Write-Host "Please ensure MongoDB is properly installed and added to PATH"
}

Write-Host ""
Write-Host "4. Testing CLI tool..." -ForegroundColor Yellow
Write-Host "Testing basic functionality (without MongoDB)..." -ForegroundColor Cyan
try {
    $result = cargo run -- --date 2025-08-15 --company 2330 --format json 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "[OK] CLI tool basic functionality works" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] CLI tool test failed" -ForegroundColor Red
    }
} catch {
    Write-Host "[ERROR] Could not test CLI tool" -ForegroundColor Red
}

Write-Host ""
Write-Host "Setup Summary:" -ForegroundColor Green
Write-Host "=============" -ForegroundColor Green
if ($mongodPath -and $service -and $service.Status -eq "Running") {
    Write-Host "[READY] You can use MongoDB features!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Test MongoDB functionality:" -ForegroundColor Cyan
    Write-Host "cargo run -- --date 2025-08-15 --save-mongodb" -ForegroundColor White
    Write-Host ""
    Write-Host "Test with specific company:" -ForegroundColor Cyan
    Write-Host "cargo run -- --date 2025-08-15 --company 2330 --save-mongodb --format json" -ForegroundColor White
} else {
    Write-Host "[INFO] MongoDB not fully ready, but CLI tool works without it!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "You can still use all other features:" -ForegroundColor Cyan
    Write-Host "cargo run -- --date 2025-08-15 --format json" -ForegroundColor White
    Write-Host "cargo run -- --date 2025-08-15 --company 2330 --format txt" -ForegroundColor White
}

Write-Host ""
Write-Host "Press any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
