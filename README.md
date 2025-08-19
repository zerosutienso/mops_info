# 🏢 證交所重大訊息擷取系統

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MongoDB](https://img.shields.io/badge/MongoDB-4.4+-green.svg)](https://www.mongodb.com/)

一個功能完整的台灣證券交易所重大訊息擷取與分析系統，使用 Rust 開發，提供高效能的資料擷取、儲存和查詢功能。

## ✨ 主要功能

### 📊 資料擷取
- 🔍 **智能解析**：自動擷取重大訊息詳細內容
- 📅 **事實發生日**：從隱藏欄位擷取標準化日期格式
- 📋 **條款代號**：自動識別並對應 51 個條款代號說明
- 💾 **多格式輸出**：支援 JSON、TXT、HTML 格式
- 🔄 **自動備份**：每次查詢自動儲存原始 HTML

### 🗄️ 資料庫整合
- 📈 **MongoDB 支援**：高效能資料儲存與查詢
- 🚫 **防重複機制**：智能去重，支援 upsert 和 skip 模式
- 🔍 **索引優化**：針對常用查詢建立索引
- 📊 **統計分析**：內建資料統計和分析功能

### 🌐 Web 介面
- 🖥️ **現代化 UI**：響應式設計，支援桌面和行動裝置
- 🔍 **即時搜尋**：公司代號、名稱、標題即時過濾
- 📅 **日期範圍查詢**：靈活的日期區間選擇
- 📤 **CSV 匯出**：一鍵匯出查詢結果
- 🏷️ **條款標籤**：視覺化顯示條款代號和說明

## 🚀 快速開始

### 1. 環境需求
- **Rust**: 1.70 或更新版本
- **MongoDB**: 4.4 或更新版本（可選）
- **作業系統**: Windows、macOS、Linux

### 2. 安裝與建置
```bash
# 克隆專案
git clone <repository-url>
cd 證交所重大訊息_augment

# 建置專案
cargo build --release
```

### 3. 基本使用
```bash
# 查詢指定日期的重大訊息
./target/release/twse-announcements.exe --date 2025-08-18

# 查詢並儲存到 MongoDB
./target/release/twse-announcements.exe --date 2025-08-18 --save-mongodb

# 啟動 Web 查看器
./target/release/simple_web_server.exe
```

## 📖 詳細文件

### 📋 功能說明
- [條款代號對照表功能](./條款代號對照表功能說明.md)
- [事實發生日擷取功能](./事實發生日擷取功能說明.md)
- [自動儲存原始HTML功能](./自動儲存原始HTML功能說明.md)
- [日期範圍查詢功能](./日期範圍查詢功能說明.md)

### 🛠️ 設定與安裝
- [MongoDB 快速安裝指南](./MongoDB快速安裝指南.md)
- [MongoDB 設定說明](./MongoDB設定說明.md)
- [無 MongoDB 使用指南](./無MongoDB使用指南.md)

### 💻 使用指南
- [Web 查看器使用說明](./Web查看器使用說明.md)
- [MongoDB 使用範例](./MongoDB使用範例.md)
- [完整功能展示](./完整功能展示.md)

### 🔧 開發文件
- [MongoDB 整合總結](./MongoDB整合總結.md)
- [Web 查看器開發總結](./Web查看器開發總結.md)
- [最終專案總結](./最終專案總結.md)

## 🎯 使用範例

### 命令列工具
```bash
# 基本查詢
./target/release/twse-announcements.exe --date 2025-08-18

# 輸出 JSON 格式
./target/release/twse-announcements.exe --date 2025-08-18 --format json

# 儲存到 MongoDB
./target/release/twse-announcements.exe --date 2025-08-18 --save-mongodb

# 日期範圍查詢
./target/release/twse-announcements.exe --start-date 2025-08-15 --end-date 2025-08-18
```

### Web 介面
```bash
# 啟動 Web 服務器
./target/release/simple_web_server.exe

# 開啟瀏覽器訪問
http://127.0.0.1:3000
```

## 📊 資料格式

### JSON 輸出範例
```json
{
  "company_code": "2327",
  "company_name": "國巨",
  "title": "公告本公司股票面額變更",
  "date": "114/08/18",
  "time": "16:30:15",
  "fact_occurrence_date": "2025-07-08",
  "clause_code": "51",
  "detail_content": "詳細內容...",
  "created_at": "2025-08-18T14:27:53.394Z"
}
```

### MongoDB 文件結構
```javascript
{
  "_id": ObjectId("..."),
  "company_code": "2327",
  "company_name": "國巨",
  "title": "公告本公司股票面額變更",
  "fact_occurrence_date": "2025-07-08",
  "clause_code": "51",
  "created_at": ISODate("2025-08-18T14:27:53.394Z"),
  "query_date": "2025-08-18"
}
```

## 🏗️ 專案結構

```
證交所重大訊息_augment/
├── src/
│   ├── main.rs              # 主程式
│   ├── simple_web.rs        # Web 服務器
│   └── web_viewer.rs        # Web 查看器
├── templates/               # HTML 模板
├── docs/                    # 文件目錄
├── examples/               # 使用範例
└── scripts/                # 輔助腳本
```

## 🔧 設定選項

### 命令列參數
- `--date`: 查詢日期 (YYYY-MM-DD)
- `--start-date`: 起始日期
- `--end-date`: 結束日期
- `--format`: 輸出格式 (json/text)
- `--save-mongodb`: 儲存到 MongoDB
- `--duplicate-mode`: 重複處理模式 (upsert/skip)

### 環境變數
- `MONGODB_URI`: MongoDB 連線字串
- `DATABASE_NAME`: 資料庫名稱
- `COLLECTION_NAME`: 集合名稱

## 🤝 貢獻指南

歡迎提交 Issue 和 Pull Request！

1. Fork 專案
2. 創建功能分支
3. 提交變更
4. 推送到分支
5. 創建 Pull Request

## 📄 授權

本專案採用 MIT 授權條款 - 詳見 [LICENSE](LICENSE) 檔案

## 🙏 致謝

- 台灣證券交易所提供公開資料
- Rust 社群的優秀套件
- MongoDB 團隊的資料庫技術

---

**📞 聯絡資訊**
如有問題或建議，歡迎開啟 Issue 或聯絡專案維護者。
