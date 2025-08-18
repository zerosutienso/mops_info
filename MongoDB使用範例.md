# MongoDB 使用範例

## 🚀 快速開始

### 1. 安裝 MongoDB
```bash
# 執行自動設定腳本
mongodb_setup.bat
```

### 2. 基本使用
```bash
# 查詢當日資料並儲存到 MongoDB
cargo run -- --save-mongodb

# 查詢指定日期並儲存到 MongoDB
cargo run -- --date 2025-08-15 --save-mongodb

# 查詢特定公司並儲存到 MongoDB
cargo run -- --date 2025-08-15 --company 2330 --save-mongodb
```

## 📊 實際測試範例

### 範例 1：儲存台積電資料
```bash
cargo run -- --date 2025-08-15 --company 2330 --save-mongodb --format json
```

**預期輸出**：
```
查詢日期: 2025-08-15
正在連接 MongoDB...
MongoDB 索引建立完成
成功儲存 1 筆資料到 MongoDB
資料庫: twse_db, 集合: announcements
[JSON 資料...]
JSON 檔案已儲存: twse_announcements_20250815.json
```

### 範例 2：儲存國泰金資料
```bash
cargo run -- --date 2025-08-15 --company 2882 --save-mongodb
```

**預期輸出**：
```
查詢日期: 2025-08-15
正在連接 MongoDB...
MongoDB 索引建立完成
成功儲存 13 筆資料到 MongoDB
資料庫: twse_db, 集合: announcements
[表格資料...]
TXT 檔案已儲存: twse_announcements_20250815.txt
```

### 範例 3：儲存當日所有資料
```bash
cargo run -- --date 2025-08-15 --save-mongodb
```

**預期輸出**：
```
查詢日期: 2025-08-15
正在連接 MongoDB...
MongoDB 索引建立完成
成功儲存 400+ 筆資料到 MongoDB
資料庫: twse_db, 集合: announcements
[表格資料...]
TXT 檔案已儲存: twse_announcements_20250815.txt
```

## 🔍 MongoDB 查詢範例

### 使用 MongoDB Compass（圖形化介面）

1. **連接資料庫**
   - 開啟 MongoDB Compass
   - 連接到 `mongodb://localhost:27017`
   - 選擇資料庫 `twse_db`
   - 選擇集合 `announcements`

2. **基本查詢**
   ```javascript
   // 查詢台積電的資料
   {"company_code": "2330"}
   
   // 查詢特定日期的資料
   {"query_date": "2025-08-15"}
   
   // 查詢包含特定關鍵字的標題
   {"title": {"$regex": "財務報告", "$options": "i"}}
   ```

### 使用 MongoDB Shell

```bash
# 啟動 MongoDB Shell
mongosh

# 切換到資料庫
use twse_db

# 查看集合
show collections

# 查詢範例
```

```javascript
// 1. 查詢所有台積電的公告
db.announcements.find({"company_code": "2330"}).pretty()

// 2. 查詢特定日期的所有公告
db.announcements.find({"query_date": "2025-08-15"}).pretty()

// 3. 統計各公司的公告數量
db.announcements.aggregate([
  {$group: {_id: "$company_code", count: {$sum: 1}, company_name: {$first: "$company_name"}}},
  {$sort: {count: -1}},
  {$limit: 10}
])

// 4. 查詢最新的 10 筆公告
db.announcements.find().sort({"created_at": -1}).limit(10).pretty()

// 5. 查詢包含特定關鍵字的公告
db.announcements.find({"title": {$regex: "財務報告", $options: "i"}}).pretty()

// 6. 查詢特定時間範圍的公告
db.announcements.find({
  "created_at": {
    $gte: ISODate("2025-08-15T00:00:00Z"),
    $lt: ISODate("2025-08-16T00:00:00Z")
  }
}).pretty()

// 7. 統計每日公告數量
db.announcements.aggregate([
  {$group: {_id: "$query_date", count: {$sum: 1}}},
  {$sort: {"_id": -1}}
])
```

## 📈 進階查詢範例

### 1. 分析公司活躍度
```javascript
// 查詢最活躍的公司（公告數量最多）
db.announcements.aggregate([
  {
    $group: {
      _id: {
        company_code: "$company_code",
        company_name: "$company_name"
      },
      total_announcements: {$sum: 1},
      latest_announcement: {$max: "$created_at"}
    }
  },
  {$sort: {total_announcements: -1}},
  {$limit: 20}
])
```

### 2. 時間趨勢分析
```javascript
// 分析每小時的公告分布
db.announcements.aggregate([
  {
    $project: {
      hour: {$hour: "$created_at"},
      company_code: 1,
      title: 1
    }
  },
  {
    $group: {
      _id: "$hour",
      count: {$sum: 1}
    }
  },
  {$sort: {"_id": 1}}
])
```

### 3. 關鍵字分析
```javascript
// 分析標題中的關鍵字頻率
db.announcements.aggregate([
  {
    $project: {
      keywords: {
        $cond: [
          {$regexMatch: {input: "$title", regex: "財務報告"}},
          "財務報告",
          {
            $cond: [
              {$regexMatch: {input: "$title", regex: "董事會"}},
              "董事會",
              {
                $cond: [
                  {$regexMatch: {input: "$title", regex: "股利"}},
                  "股利",
                  "其他"
                ]
              }
            ]
          }
        ]
      }
    }
  },
  {
    $group: {
      _id: "$keywords",
      count: {$sum: 1}
    }
  },
  {$sort: {count: -1}}
])
```

## 🛠️ 資料管理

### 1. 清理測試資料
```javascript
// 刪除特定日期的資料
db.announcements.deleteMany({"query_date": "2025-08-15"})

// 刪除所有資料
db.announcements.deleteMany({})

// 刪除整個集合
db.announcements.drop()
```

### 2. 建立索引
```javascript
// 手動建立索引（程式會自動建立）
db.announcements.createIndex({"company_code": 1})
db.announcements.createIndex({"query_date": 1})
db.announcements.createIndex({"created_at": 1})
db.announcements.createIndex({"date": 1, "time": 1})

// 查看索引
db.announcements.getIndexes()
```

### 3. 資料統計
```javascript
// 查看集合統計
db.announcements.stats()

// 查看資料庫大小
db.stats()

// 計算總記錄數
db.announcements.countDocuments()
```

## 🔄 自動化腳本

### 每日資料收集腳本
```batch
@echo off
echo 開始收集今日重大訊息...
cargo run -- --save-mongodb
echo 資料收集完成！
```

### 歷史資料回補腳本
```batch
@echo off
echo 開始回補歷史資料...
for /L %%i in (1,1,30) do (
    echo 正在處理 2025-08-%%i...
    cargo run -- --date 2025-08-%%i --save-mongodb
)
echo 歷史資料回補完成！
```

## 🎯 實用技巧

### 1. 效能優化
- 使用索引加速查詢
- 限制查詢結果數量
- 使用投影只返回需要的欄位

### 2. 資料備份
```bash
# 備份整個資料庫
mongodump --db twse_db --out backup/

# 還原資料庫
mongorestore --db twse_db backup/twse_db/
```

### 3. 監控和維護
```javascript
// 查看當前連接
db.runCommand({currentOp: true})

// 查看資料庫狀態
db.runCommand({dbStats: 1})

// 查看集合狀態
db.runCommand({collStats: "announcements"})
```

這樣你就可以充分利用 MongoDB 來管理和分析台灣證交所的重大訊息資料了！🎉
