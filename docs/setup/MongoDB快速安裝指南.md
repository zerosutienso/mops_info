# MongoDB 快速安裝指南

## 🎯 安裝步驟

### 1. 下載 MongoDB
1. 前往 [MongoDB 官方下載頁面](https://www.mongodb.com/try/download/community)
2. 選擇：
   - **Version**: 7.0.x (最新穩定版)
   - **Platform**: Windows
   - **Package**: msi
3. 點擊 "Download" 下載安裝檔

### 2. 安裝 MongoDB
1. 執行下載的 `.msi` 檔案
2. 選擇 **"Complete"** 安裝類型
3. **重要**：勾選以下選項：
   - ✅ **"Install MongoDB as a Service"** (安裝為 Windows 服務)
   - ✅ **"Run service as Network Service user"** (以網路服務使用者執行)
   - ✅ **"Install MongoDB Compass"** (安裝圖形化管理工具)
4. 點擊 "Install" 開始安裝

### 3. 驗證安裝
開啟 **命令提示字元** 或 **PowerShell**，執行：

```bash
# 檢查 MongoDB 服務狀態
sc query MongoDB

# 檢查 mongod 是否可用
mongod --version

# 檢查 mongosh 是否可用
mongosh --version
```

## 🚀 快速測試

### 方法 1：使用我們的測試腳本
```bash
# 在專案目錄執行
.\mongodb_setup.bat
```

### 方法 2：手動測試
```bash
# 啟動 MongoDB 服務（如果未啟動）
net start MongoDB

# 連接到 MongoDB
mongosh

# 在 MongoDB shell 中測試
db.runCommand({ping: 1})
```

## 🛠️ 常見問題解決

### 問題 1：找不到 mongod 指令
**原因**：MongoDB 未加入系統 PATH

**解決方法**：
1. 找到 MongoDB 安裝目錄（通常在 `C:\Program Files\MongoDB\Server\7.0\bin`）
2. 將此目錄加入系統 PATH：
   - 開啟「系統內容」→「進階」→「環境變數」
   - 在「系統變數」中找到「Path」，點擊「編輯」
   - 點擊「新增」，輸入 MongoDB bin 目錄路徑
   - 點擊「確定」儲存

### 問題 2：MongoDB 服務無法啟動
**解決方法**：
```bash
# 以管理員身分執行命令提示字元
net start MongoDB

# 或手動啟動服務
sc start MongoDB
```

### 問題 3：連接被拒絕
**檢查項目**：
1. MongoDB 服務是否正在運行
2. 防火牆是否阻擋 27017 埠
3. MongoDB 設定檔是否正確

## 🎯 測試 CLI 工具

安裝完成後，測試我們的 CLI 工具：

```bash
# 基本測試（不使用 MongoDB）
cargo run -- --date 2025-08-15 --company 2330

# MongoDB 功能測試
cargo run -- --date 2025-08-15 --company 2330 --save-mongodb
```

## 📊 MongoDB Compass 使用

MongoDB Compass 是圖形化管理工具：

1. **啟動 Compass**：從開始選單找到 "MongoDB Compass"
2. **連接資料庫**：使用預設連接字串 `mongodb://localhost:27017`
3. **查看資料**：
   - 選擇資料庫：`twse_db`
   - 選擇集合：`announcements`
   - 瀏覽和查詢資料

## 🔧 進階設定

### 自訂資料目錄
如果要自訂資料儲存位置：

1. 建立資料目錄：`mkdir C:\mongodb\data`
2. 啟動 MongoDB：`mongod --dbpath C:\mongodb\data`

### 設定檔案
MongoDB 設定檔通常位於：
`C:\Program Files\MongoDB\Server\7.0\bin\mongod.cfg`

## 📝 安裝檢查清單

安裝完成後，確認以下項目：

- [ ] MongoDB 服務已安裝並運行
- [ ] `mongod --version` 指令可執行
- [ ] `mongosh --version` 指令可執行
- [ ] MongoDB Compass 可正常啟動
- [ ] 可連接到 `mongodb://localhost:27017`
- [ ] 我們的 CLI 工具可正常使用 `--save-mongodb` 參數

## 🎉 完成！

安裝完成後，你就可以：

1. **儲存資料到 MongoDB**：
   ```bash
   cargo run -- --save-mongodb
   ```

2. **查詢和分析資料**：
   ```bash
   mongosh
   use twse_db
   db.announcements.find().limit(5)
   ```

3. **使用圖形化介面**：
   開啟 MongoDB Compass 瀏覽資料

現在你擁有了完整的台灣證交所重大訊息資料管理系統！🚀

## 📞 需要幫助？

如果遇到問題：
1. 檢查 MongoDB 官方文件
2. 確認 Windows 版本相容性
3. 檢查防毒軟體是否阻擋安裝
4. 以管理員身分執行安裝程式
