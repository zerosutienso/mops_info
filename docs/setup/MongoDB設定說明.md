# MongoDB 整合設定說明

## 🎯 功能概述

我們的 CLI 工具現在支援將查詢結果直接儲存到 MongoDB 資料庫中，讓你可以：
- 建立歷史資料庫
- 進行複雜的資料分析
- 建立 API 服務
- 實現資料視覺化

## 📦 MongoDB 安裝

### Windows 安裝
1. 下載 MongoDB Community Server：https://www.mongodb.com/try/download/community
2. 執行安裝程式，選擇 "Complete" 安裝
3. 勾選 "Install MongoDB as a Service"
4. 勾選 "Install MongoDB Compass"（圖形化管理工具）

### 啟動 MongoDB 服務
```bash
# Windows (以管理員身分執行)
net start MongoDB

# 或使用 mongod 直接啟動
mongod --dbpath "C:\data\db"
```

### 驗證安裝
```bash
# 連接到 MongoDB
mongosh

# 在 MongoDB shell 中
show dbs
```

## 🚀 使用方式

### 基本用法
```bash
# 查詢當日資料並儲存到 MongoDB
cargo run -- --save-mongodb

# 查詢指定日期並儲存到 MongoDB
cargo run -- --date 2025-08-15 --save-mongodb

# 自訂 MongoDB 連接設定
cargo run -- --date 2025-08-15 --save-mongodb \
  --mongodb-uri "mongodb://localhost:27017" \
  --mongodb-database "twse_db" \
  --mongodb-collection "announcements"
```

### 進階用法
```bash
# 同時輸出 JSON 和儲存到 MongoDB
cargo run -- --date 2025-08-15 --format json --save-mongodb

# 篩選特定公司並儲存到 MongoDB
cargo run -- --date 2025-08-15 --company 2330 --save-mongodb

# 使用自訂資料庫和集合名稱
cargo run -- --date 2025-08-15 --save-mongodb \
  --mongodb-database "financial_data" \
  --mongodb-collection "daily_announcements"
```

## 📊 資料結構

### MongoDB 文件結構
```json
{
  "_id": ObjectId("..."),
  "company_code": "2330",
  "company_name": "台積電",
  "title": "本公司代子公司 TSMC Global Ltd. 公告取得固定收益證券",
  "date": "114/08/15",
  "time": "17:40:31",
  "detail_content": "1.證券名稱:\n公司債。\n2.交易日期:114/8/12~114/8/15...",
  "announcement_type": "符合條款第四條第XX款：12",
  "fact_date": "114/08/15",
  "created_at": ISODate("2025-08-17T10:30:00.000Z"),
  "query_date": "2025-08-15"
}
```

### 欄位說明
- `_id`: MongoDB 自動生成的唯一識別碼
- `company_code`: 公司代號
- `company_name`: 公司名稱
- `title`: 重大訊息標題
- `date`: 發言日期（民國年格式）
- `time`: 發言時間
- `detail_content`: 完整詳細內容
- `announcement_type`: 公告類型
- `fact_date`: 事實發生日期
- `created_at`: 資料建立時間（UTC）
- `query_date`: 查詢日期（西元年格式）

## 🔍 MongoDB 查詢範例

### 使用 MongoDB Compass（圖形化介面）
1. 開啟 MongoDB Compass
2. 連接到 `mongodb://localhost:27017`
3. 選擇資料庫 `twse_db`
4. 選擇集合 `announcements`
5. 使用篩選器查詢資料

### 使用 MongoDB Shell
```javascript
// 連接到資料庫
use twse_db

// 查看所有集合
show collections

// 查詢特定公司的資料
db.announcements.find({"company_code": "2330"})

// 查詢特定日期的資料
db.announcements.find({"query_date": "2025-08-15"})

// 查詢包含特定關鍵字的標題
db.announcements.find({"title": {$regex: "財務報告", $options: "i"}})

// 統計各公司的公告數量
db.announcements.aggregate([
  {$group: {_id: "$company_code", count: {$sum: 1}}},
  {$sort: {count: -1}}
])

// 查詢最近的公告
db.announcements.find().sort({"created_at": -1}).limit(10)
```

## 📈 索引設定

程式會自動建立以下索引以提升查詢效能：
- `company_code`: 公司代號索引
- `query_date`: 查詢日期索引
- `date, time`: 發言日期時間複合索引
- `created_at`: 建立時間索引

## 🔧 設定參數

### 命令列參數
- `--save-mongodb`: 啟用 MongoDB 儲存功能
- `--mongodb-uri`: MongoDB 連接字串（預設：mongodb://localhost:27017）
- `--mongodb-database`: 資料庫名稱（預設：twse_db）
- `--mongodb-collection`: 集合名稱（預設：announcements）

### 環境變數（可選）
```bash
# 設定環境變數
export MONGODB_URI="mongodb://localhost:27017"
export MONGODB_DATABASE="twse_db"
export MONGODB_COLLECTION="announcements"
```

## 🛠️ 故障排除

### 常見問題

#### 1. 連接失敗
```
Error: Failed to connect to MongoDB
```
**解決方法**：
- 確認 MongoDB 服務已啟動
- 檢查連接字串是否正確
- 確認防火牆設定

#### 2. 權限問題
```
Error: Authentication failed
```
**解決方法**：
- 檢查使用者名稱和密碼
- 確認使用者有寫入權限

#### 3. 資料庫不存在
程式會自動建立資料庫和集合，無需手動建立。

### 檢查 MongoDB 狀態
```bash
# 檢查 MongoDB 服務狀態
sc query MongoDB

# 檢查 MongoDB 程序
tasklist | findstr mongod
```

## 📊 資料分析範例

### 使用 MongoDB Aggregation Pipeline
```javascript
// 分析各公司公告數量趨勢
db.announcements.aggregate([
  {
    $group: {
      _id: {
        company_code: "$company_code",
        company_name: "$company_name",
        query_date: "$query_date"
      },
      count: {$sum: 1}
    }
  },
  {
    $sort: {"_id.query_date": -1, "count": -1}
  }
])

// 分析公告類型分布
db.announcements.aggregate([
  {
    $group: {
      _id: "$announcement_type",
      count: {$sum: 1}
    }
  },
  {
    $sort: {count: -1}
  }
])
```

## 🔄 資料備份與還原

### 備份資料
```bash
# 備份整個資料庫
mongodump --db twse_db --out backup/

# 備份特定集合
mongodump --db twse_db --collection announcements --out backup/
```

### 還原資料
```bash
# 還原整個資料庫
mongorestore --db twse_db backup/twse_db/

# 還原特定集合
mongorestore --db twse_db --collection announcements backup/twse_db/announcements.bson
```

## 🎯 下一步建議

1. **建立定期任務**：使用 Windows 工作排程器定期執行查詢
2. **建立 Web API**：基於 MongoDB 資料建立 REST API
3. **資料視覺化**：使用 MongoDB Charts 或其他工具進行視覺化
4. **資料分析**：使用 Python 或 R 連接 MongoDB 進行深度分析

這樣你就有了一個完整的台灣證交所重大訊息資料庫系統！🎉
