# 🔧 腳本工具

本目錄包含證交所重大訊息擷取系統的各種輔助腳本和工具。

## 📋 腳本列表

### 🌐 Web 服務器腳本
- **`start_simple_web.bat`** - 啟動簡單 Web 服務器
- **`start_web_viewer.bat`** - 啟動 Web 查看器
- **`start_optimized_web.bat`** - 啟動優化版 Web 服務器

### 🗄️ MongoDB 相關腳本
- **`mongodb_setup.bat`** - MongoDB 自動安裝和設定
- **`check_mongodb.ps1`** - 檢查 MongoDB 服務狀態

### 🧪 測試腳本
- **`test_web_server.bat`** - 測試 Web 服務器功能
- **`test_date_query.bat`** - 測試日期查詢功能

### 📊 範例腳本
- **`examples.bat`** - 各種使用範例的示範腳本

## 🚀 使用方式

### Web 服務器啟動

#### 簡單 Web 服務器
```batch
# 啟動基本的 Web API 服務器
scripts\start_simple_web.bat
```
- 提供 REST API 介面
- 支援 JSON 資料查詢
- 輕量級，適合 API 使用

#### Web 查看器
```batch
# 啟動完整的 Web 查看器
scripts\start_web_viewer.bat
```
- 完整的 Web UI 介面
- 支援即時搜尋和過濾
- 支援 CSV 匯出功能

#### 優化版 Web 服務器
```batch
# 啟動優化版 Web 服務器
scripts\start_optimized_web.bat
```
- 效能優化版本
- 支援大量資料查詢
- 適合生產環境使用

### MongoDB 設定

#### 自動安裝 MongoDB
```batch
# 一鍵安裝和設定 MongoDB
scripts\mongodb_setup.bat
```
功能：
- 自動下載 MongoDB
- 建立資料目錄
- 啟動 MongoDB 服務
- 建立必要的索引

#### 檢查 MongoDB 狀態
```powershell
# 檢查 MongoDB 服務狀態
scripts\check_mongodb.ps1
```
功能：
- 檢查服務是否運行
- 測試資料庫連線
- 顯示資料庫統計資訊

### 測試工具

#### Web 服務器測試
```batch
# 測試 Web 服務器各項功能
scripts\test_web_server.bat
```
測試項目：
- API 端點回應
- 資料查詢功能
- 錯誤處理機制

#### 日期查詢測試
```batch
# 測試日期查詢功能
scripts\test_date_query.bat
```
測試項目：
- 單日查詢
- 日期範圍查詢
- 錯誤日期處理

### 使用範例

#### 執行範例腳本
```batch
# 執行各種功能的示範
scripts\examples.bat
```
包含範例：
- 基本資料擷取
- MongoDB 儲存
- Web 查詢
- CSV 匯出

## 🔧 腳本詳細說明

### start_simple_web.bat
```batch
@echo off
echo 啟動簡單 Web 服務器...
cd /d "%~dp0.."
.\target\release\simple_web_server.exe
pause
```

### start_web_viewer.bat
```batch
@echo off
echo 啟動 Web 查看器...
cd /d "%~dp0.."
.\target\release\web_viewer.exe
pause
```

### mongodb_setup.bat
```batch
@echo off
echo MongoDB 自動安裝腳本
echo 正在檢查 MongoDB 安裝狀態...
# 完整的安裝邏輯
```

### check_mongodb.ps1
```powershell
# MongoDB 狀態檢查腳本
Write-Host "檢查 MongoDB 服務狀態..." -ForegroundColor Green
# 完整的檢查邏輯
```

## 📊 腳本功能對照表

| 腳本名稱 | 功能 | 適用場景 | 依賴 |
|---------|------|----------|------|
| start_simple_web.bat | 啟動 API 服務器 | API 開發、測試 | 編譯完成的執行檔 |
| start_web_viewer.bat | 啟動 Web UI | 資料查看、分析 | 編譯完成的執行檔 |
| start_optimized_web.bat | 啟動優化服務器 | 生產環境 | 編譯完成的執行檔 |
| mongodb_setup.bat | 安裝 MongoDB | 初次設定 | 網路連線 |
| check_mongodb.ps1 | 檢查 MongoDB | 故障排除 | PowerShell |
| test_web_server.bat | 測試 Web 功能 | 功能驗證 | curl 或類似工具 |
| test_date_query.bat | 測試查詢功能 | 功能驗證 | 編譯完成的執行檔 |
| examples.bat | 功能示範 | 學習、展示 | 編譯完成的執行檔 |

## 🛠️ 自訂腳本

### 建立自訂腳本
你可以參考現有腳本建立自己的自動化腳本：

```batch
@echo off
echo 我的自訂腳本
cd /d "%~dp0.."

# 你的命令
.\target\release\twse-announcements.exe --date 2025-08-18 --save-mongodb

echo 完成！
pause
```

### 腳本最佳實踐
1. **錯誤處理**: 加入適當的錯誤檢查
2. **路徑處理**: 使用相對路徑確保可移植性
3. **使用者提示**: 提供清楚的執行狀態訊息
4. **暫停機制**: 加入 `pause` 讓使用者查看結果

## 🔍 故障排除

### 常見問題

#### 腳本無法執行
```batch
# 檢查執行檔是否存在
dir target\release\*.exe

# 重新建置專案
cargo build --release
```

#### MongoDB 連線失敗
```batch
# 檢查 MongoDB 服務
scripts\check_mongodb.ps1

# 重新啟動 MongoDB
net stop MongoDB
net start MongoDB
```

#### Web 服務器無法啟動
```batch
# 檢查埠號是否被佔用
netstat -an | findstr :3000

# 使用不同埠號
set PORT=3001
scripts\start_simple_web.bat
```

## 📝 維護指南

### 定期維護
- 檢查腳本是否與最新版本相容
- 更新路徑和參數設定
- 測試所有腳本功能

### 版本控制
- 腳本變更時更新版本註釋
- 保留舊版本作為備份
- 記錄變更歷史

---

**💡 提示**: 建議先執行測試腳本確認環境設定正確，再使用其他功能腳本。
