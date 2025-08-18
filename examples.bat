@echo off
echo 台灣證交所重大訊息查詢 CLI 使用範例
echo =====================================
echo.

echo 1. 查詢今日重大訊息（表格格式）
cargo run
echo.

echo 2. 查詢今日重大訊息（JSON格式）
cargo run -- --format json
echo.

echo 3. 查詢指定日期的重大訊息（2025-08-15，有資料）
cargo run -- --date 2025-08-15
echo.

echo 4. 查詢特定公司的重大訊息
cargo run -- --company 2330 --date 2025-08-15
echo.

echo 5. 儲存原始 HTML 和所有格式
cargo run -- --date 2025-08-15 --save-html --format json
echo.

echo 6. 輸出 TXT 格式
cargo run -- --date 2025-08-15 --format txt
echo.

echo 7. 顯示幫助資訊
cargo run -- --help
echo.

echo 範例執行完畢！
pause
