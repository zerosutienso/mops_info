@echo off
echo MongoDB Setup and Test Script
echo =============================
echo.

echo 1. Checking if MongoDB is installed...
where mongod >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] MongoDB is installed
    mongod --version 2>nul | findstr "db version"
) else (
    echo [ERROR] MongoDB is not installed
    echo.
    echo Please follow these steps to install MongoDB:
    echo 1. Go to https://www.mongodb.com/try/download/community
    echo 2. Download MongoDB Community Server for Windows
    echo 3. Run the installer and choose "Complete" installation
    echo 4. Check "Install MongoDB as a Service"
    echo 5. Check "Install MongoDB Compass" (GUI management tool)
    echo.
    echo Do you want to open the download page? (y/n)
    set /p choice=
    if /i "%choice%"=="y" start https://www.mongodb.com/try/download/community
    echo.
    echo Please re-run this script after installation
    pause
    exit /b 1
)
echo.

echo 2. Checking MongoDB service status...
sc query MongoDB >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] MongoDB service is installed
    sc query MongoDB | findstr "STATE"
) else (
    echo [ERROR] MongoDB service is not installed
    echo Please ensure "Install MongoDB as a Service" was checked during installation
)
echo.

echo 3. Attempting to start MongoDB service...
echo Starting MongoDB service...
net start MongoDB >nul 2>&1
if %errorlevel% equ 0 (
    echo [OK] MongoDB service started successfully
) else (
    echo [WARNING] MongoDB service may already be running or requires admin privileges
)
echo.

echo 4. Testing MongoDB connection...
echo Testing MongoDB connection...
where mongosh >nul 2>&1
if %errorlevel% equ 0 (
    mongosh --eval "db.runCommand({ping: 1})" --quiet >nul 2>&1
    if !errorlevel! equ 0 (
        echo [OK] MongoDB connection test successful
    ) else (
        echo [ERROR] MongoDB connection test failed
        echo Please check if MongoDB is running
    )
) else (
    echo [WARNING] mongosh not found, trying mongo command...
    where mongo >nul 2>&1
    if !errorlevel! equ 0 (
        mongo --eval "db.runCommand({ping: 1})" --quiet >nul 2>&1
        if !errorlevel! equ 0 (
            echo [OK] MongoDB connection test successful (using legacy mongo command)
        ) else (
            echo [ERROR] MongoDB connection test failed
            echo Please check if MongoDB is running
        )
    ) else (
        echo [ERROR] MongoDB client tools not found
        echo Please ensure MongoDB is properly installed and added to PATH
    )
)
echo.

echo 5. Creating test database and collection...
echo Creating test database...
where mongosh >nul 2>&1
if %errorlevel% equ 0 (
    mongosh --eval "use twse_db; db.test.insertOne({test: 'data'}); db.test.drop();" --quiet >nul 2>&1
    if !errorlevel! equ 0 (
        echo [OK] Test database created successfully
    ) else (
        echo [ERROR] Test database creation failed
    )
) else (
    where mongo >nul 2>&1
    if !errorlevel! equ 0 (
        mongo --eval "use twse_db; db.test.insertOne({test: 'data'}); db.test.drop();" --quiet >nul 2>&1
        if !errorlevel! equ 0 (
            echo [OK] Test database created successfully (using legacy mongo command)
        ) else (
            echo [ERROR] Test database creation failed
        )
    ) else (
        echo [WARNING] Skipping database test (client tools not found)
    )
)
echo.

echo MongoDB setup completed!
echo.
echo You can now test the CLI tool's MongoDB functionality with:
echo.
echo Basic test:
echo cargo run -- --date 2025-08-15 --save-mongodb
echo.
echo Advanced test:
echo cargo run -- --date 2025-08-15 --company 2330 --save-mongodb --format json
echo.
pause
