@echo off
echo 測試 Web 服務器啟動...
echo.

echo 正在檢查 MongoDB 連接...
mongosh --eval "db.runCommand({ping: 1})" --quiet >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] 無法連接到 MongoDB
    pause
    exit /b 1
)
echo [OK] MongoDB 連接正常

echo.
echo 正在啟動 Web 服務器...
echo 位址: http://127.0.0.1:3000
echo.

.\target\release\simple_web_server.exe
echo.
echo Web 服務器已停止
pause
