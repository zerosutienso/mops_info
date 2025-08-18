@echo off
echo ========================================
echo 台灣證交所重大訊息優化 Web 查看器
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
for /f %%i in ('mongosh --eval "use twse_db; db.announcements.countDocuments({})" --quiet 2^>nul') do set count=%%i
if "%count%"=="0" (
    echo [WARNING] 資料庫中沒有資料
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
        echo 將啟動空的 Web 查看器，您可以稍後添加資料
    )
) else (
    echo [OK] 找到 %count% 筆資料
)

echo.

echo 正在編譯優化版 Web 查看器...
cargo build --bin simple_web_server --release
if %errorlevel% neq 0 (
    echo [ERROR] 編譯失敗
    pause
    exit /b 1
)

echo.
echo 正在啟動優化版 Web 查看器...
echo.
echo ========================================
echo 🚀 優化版 Web 查看器啟動中...
echo 📍 位址: http://127.0.0.1:3000
echo 🔗 請在瀏覽器中開啟上述網址
echo 🔌 API: http://127.0.0.1:3000/api/announcements
echo 📊 統計: http://127.0.0.1:3000/api/stats
echo ⏹️  按 Ctrl+C 停止服務器
echo ========================================
echo.
echo ✨ 新功能:
echo   - 公司名稱顯示在第一行
echo   - 明細內容可按按鈕切換顯示
echo   - 統計資訊面板
echo   - 展開/收合全部功能
echo   - 複製和匯出功能
echo   - 優化的卡片式設計
echo.

.\target\release\simple_web_server.exe

echo.
echo Web 查看器已停止
pause
