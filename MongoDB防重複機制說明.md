# MongoDB 防重複資料機制說明

## 🎯 概述

為了避免在 MongoDB 中儲存重複的重大訊息資料，我們實現了三種不同的防重複模式，讓使用者可以根據需求選擇最適合的處理方式。

## 🔧 三種防重複模式

### 1. Upsert 模式 (預設，推薦) ⭐

**使用方式**:
```bash
cargo run -- --date 2025-08-14 --save-mongodb --duplicate-mode upsert
# 或者省略 --duplicate-mode (預設就是 upsert)
cargo run -- --date 2025-08-14 --save-mongodb
```

**工作原理**:
- 使用 `company_code + date + time + title` 作為唯一識別
- 如果資料不存在，則新增
- 如果資料已存在，則更新為最新內容
- 保留其他不重複的資料

**適用場景**:
- ✅ 日常資料更新
- ✅ 確保資料是最新版本
- ✅ 保持資料庫整潔
- ✅ 適合自動化腳本

**輸出範例**:
```
使用 Upsert 模式：自動更新重複資料...
Upsert 操作完成:
  新增: 485 筆資料
  更新: 30 筆資料
  總計: 515 筆資料
```

### 2. Replace 模式 (完全替換)

**使用方式**:
```bash
cargo run -- --date 2025-08-14 --save-mongodb --duplicate-mode replace
```

**工作原理**:
- 刪除指定日期的所有舊資料
- 重新插入當次查詢的所有資料
- 確保該日期的資料完全是最新的

**適用場景**:
- ✅ 重新收集某日的完整資料
- ✅ 修復資料不一致問題
- ✅ 確保該日期資料的完整性
- ⚠️ 會丟失該日期的其他來源資料

**輸出範例**:
```
使用 Replace 模式：刪除舊資料後重新插入...
發現 515 筆相同日期的資料，刪除後重新插入
已刪除 515 筆舊資料
成功插入 515 筆新資料
```

### 3. Skip 模式 (跳過重複)

**使用方式**:
```bash
cargo run -- --date 2025-08-14 --save-mongodb --duplicate-mode skip
```

**工作原理**:
- 檢查每筆資料是否已存在
- 只插入不存在的新資料
- 跳過已存在的重複資料

**適用場景**:
- ✅ 增量資料收集
- ✅ 保留現有資料不變
- ✅ 只添加新發現的公告
- ✅ 多次執行不會影響現有資料

**輸出範例**:
```
使用 Skip 模式：跳過重複資料...
Skip 操作完成:
  新增: 15 筆資料
  跳過: 500 筆重複資料
  總計處理: 515 筆資料
```

## 🔍 重複判斷邏輯

### 唯一識別條件
我們使用以下四個欄位的組合來判斷資料是否重複：

```javascript
{
  "company_code": "2330",    // 公司代號
  "date": "114/08/14",       // 發言日期
  "time": "16:30:15",        // 發言時間
  "title": "公告標題..."      // 公告標題
}
```

### 為什麼選擇這些欄位？
1. **company_code**: 區分不同公司
2. **date + time**: 區分同一公司的不同公告
3. **title**: 區分同一時間的不同公告內容

這個組合確保了：
- ✅ 相同公司在相同時間的相同公告被視為重複
- ✅ 不同公司的公告不會被誤判為重複
- ✅ 同一公司的不同公告不會被誤判為重複

## 📊 使用建議

### 日常使用 (推薦 Upsert)
```bash
# 每日自動收集，使用預設的 upsert 模式
cargo run -- --save-mongodb

# 或明確指定
cargo run -- --save-mongodb --duplicate-mode upsert
```

### 歷史資料回補 (推薦 Replace)
```bash
# 重新收集特定日期的完整資料
cargo run -- --date 2025-08-14 --save-mongodb --duplicate-mode replace
```

### 增量更新 (推薦 Skip)
```bash
# 只添加新資料，不影響現有資料
cargo run -- --save-mongodb --duplicate-mode skip
```

## 🛠️ 實際使用範例

### 範例 1: 日常監控腳本
```bash
#!/bin/bash
# daily_monitor.sh
echo "開始收集今日重大訊息..."
cargo run -- --save-mongodb --duplicate-mode upsert
echo "收集完成，使用 upsert 模式確保資料最新"
```

### 範例 2: 歷史資料修復
```bash
#!/bin/bash
# fix_historical_data.sh
for date in 2025-08-10 2025-08-11 2025-08-12 2025-08-13 2025-08-14; do
    echo "重新收集 $date 的資料..."
    cargo run -- --date $date --save-mongodb --duplicate-mode replace
done
echo "歷史資料修復完成"
```

### 範例 3: 安全的增量更新
```bash
#!/bin/bash
# safe_incremental.sh
echo "安全地添加新資料，不影響現有資料..."
cargo run -- --save-mongodb --duplicate-mode skip
echo "增量更新完成"
```

## 📈 效能比較

| 模式 | 速度 | 安全性 | 資料完整性 | 適用場景 |
|------|------|--------|------------|----------|
| Upsert | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | 日常使用 |
| Replace | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | 完整重建 |
| Skip | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 增量更新 |

### 效能說明
- **Upsert**: 中等速度，每筆資料需要檢查和更新
- **Replace**: 較快，批次刪除和插入
- **Skip**: 較慢，每筆資料需要個別檢查

## 🔧 進階設定

### 自訂唯一識別邏輯
如果需要修改重複判斷邏輯，可以編輯 `src/main.rs` 中的 filter 條件：

```rust
let filter = doc! {
    "company_code": &doc.company_code,
    "date": &doc.date,
    "time": &doc.time,
    "title": &doc.title,
    // 可以添加其他欄位
};
```

### MongoDB 索引優化
系統會自動建立以下索引來提升查詢效能：
- `company_code` 索引
- `query_date` 索引
- `date, time` 複合索引
- `created_at` 索引

## 🚨 注意事項

### Replace 模式警告
- ⚠️ 會刪除指定日期的所有資料
- ⚠️ 如果該日期有其他來源的資料也會被刪除
- ⚠️ 建議在使用前先備份資料

### Skip 模式限制
- ⚠️ 不會更新已存在的資料
- ⚠️ 如果原始資料有錯誤，不會被修正
- ⚠️ 適合確定現有資料正確的情況

### Upsert 模式優勢
- ✅ 平衡了安全性和功能性
- ✅ 適合大多數使用場景
- ✅ 自動處理資料更新
- ✅ 不會丟失其他資料

## 🎯 最佳實踐

1. **日常使用**: 使用 `upsert` 模式
2. **首次收集**: 使用 `upsert` 或 `replace` 模式
3. **資料修復**: 使用 `replace` 模式
4. **謹慎更新**: 使用 `skip` 模式
5. **定期備份**: 無論使用哪種模式，都建議定期備份 MongoDB 資料

## 🎉 總結

這個防重複機制讓你可以：

1. **🔄 靈活選擇**: 三種模式適應不同需求
2. **🛡️ 避免重複**: 智能識別和處理重複資料
3. **📊 清晰回饋**: 詳細的操作結果報告
4. **⚡ 高效處理**: 優化的查詢和更新邏輯
5. **🔧 易於使用**: 簡單的命令列參數控制

現在你可以放心地多次執行資料收集，不用擔心產生重複資料了！🎊
