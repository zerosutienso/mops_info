@echo off
echo ========================================
echo 測試日期查詢功能
echo ========================================
echo.

echo 正在啟動 Web 服務器...
start /B .\target\release\simple_web_server.exe

echo 等待服務器啟動...
timeout /t 3 /nobreak >nul

echo.
echo 測試 API 端點:
echo.

echo 1. 檢查調試資訊:
echo    http://127.0.0.1:3000/api/debug
echo.

echo 2. 測試基本查詢:
echo    http://127.0.0.1:3000/api/announcements?limit=5
echo.

echo 3. 測試日期範圍查詢:
echo    http://127.0.0.1:3000/api/announcements?start_date=2025-08-14^&end_date=2025-08-15^&limit=10
echo.

echo 4. 測試統計資訊:
echo    http://127.0.0.1:3000/api/stats
echo.

echo 5. 開啟 Web 介面:
echo    http://127.0.0.1:3000
echo.

echo ========================================
echo 請在瀏覽器中測試上述網址
echo 按任意鍵開啟主要 Web 介面...
echo ========================================
pause >nul

start http://127.0.0.1:3000

echo.
echo Web 介面已開啟
echo 按任意鍵結束測試...
pause >nul

echo 正在停止服務器...
taskkill /f /im simple_web_server.exe >nul 2>&1
echo 測試完成
