@echo off
echo ========================================
echo 台灣證交所重大訊息 Web 查看器
echo ========================================
echo.

echo 正在檢查 MongoDB 連接...
mongosh --eval "db.runCommand({ping: 1})" --quiet >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] 無法連接到 MongoDB
    echo.
    echo 請確認:
    echo 1. MongoDB 服務正在運行
    echo 2. 使用預設連接 mongodb://localhost:27017
    echo.
    echo 如需安裝 MongoDB，請參考 MongoDB快速安裝指南.md
    echo.
    pause
    exit /b 1
)

echo [OK] MongoDB 連接正常
echo.

echo 正在檢查資料庫資料...
mongosh --eval "use twse_db; db.announcements.countDocuments({})" --quiet >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] 找不到資料庫或集合
    echo.
    echo 請先使用 CLI 工具收集資料:
    echo cargo run -- --date 2025-08-15 --save-mongodb
    echo.
    set /p choice="是否要現在收集測試資料? (y/n): "
    if /i "%choice%"=="y" (
        echo.
        echo 正在收集 2025-08-15 的資料...
        cargo run -- --date 2025-08-15 --save-mongodb
        if %errorlevel% neq 0 (
            echo [ERROR] 資料收集失敗
            pause
            exit /b 1
        )
        echo [OK] 測試資料收集完成
    ) else (
        echo.
        echo 請手動收集資料後再啟動 Web 查看器
        pause
        exit /b 1
    )
)

echo [OK] 資料庫資料檢查完成
echo.

echo 正在編譯 Web 查看器...
cargo build --bin web_server --release
if %errorlevel% neq 0 (
    echo [ERROR] 編譯失敗
    pause
    exit /b 1
)

echo [OK] 編譯完成
echo.

echo 正在啟動 Web 查看器...
echo.
echo ========================================
echo 🚀 Web 查看器啟動中...
echo 📍 位址: http://127.0.0.1:3000
echo 🔗 請在瀏覽器中開啟上述網址
echo ⏹️  按 Ctrl+C 停止服務器
echo ========================================
echo.

cargo run --bin web_server --release

echo.
echo Web 查看器已停止
pause
