# 📁 自動儲存原始 HTML 功能說明

## 🎯 修正目標

根據你的要求，修正程式使其在每次執行查詢重大資訊時都會自動將原始 HTML 存檔，不需要額外的 `--save-html` 參數。

## ✅ 修正內容

### 1. **移除條件判斷**

#### 修正前
```rust
// 儲存原始 HTML（如果要求）
if args.save_html {
    save_html(&html_content, &filename)?;
}
```

#### 修正後
```rust
// 自動儲存原始 HTML（每次查詢都會保存）
save_html(&html_content, &filename)?;
```

### 2. **更新儲存訊息**

#### 修正前
```rust
fn save_html(html_content: &str, filename: &str) -> Result<()> {
    fs::write(format!("{}.html", filename), html_content)?;
    println!("HTML 檔案已儲存: {}.html", filename);
    Ok(())
}
```

#### 修正後
```rust
fn save_html(html_content: &str, filename: &str) -> Result<()> {
    fs::write(format!("{}.html", filename), html_content)?;
    println!("✅ 原始 HTML 檔案已自動儲存: {}.html", filename);
    Ok(())
}
```

### 3. **更新參數說明**

#### 修正前
```rust
/// 儲存原始 HTML 回應到檔案
#[arg(long)]
save_html: bool,
```

#### 修正後
```rust
/// 儲存原始 HTML 回應到檔案（已自動啟用，此參數保留向後相容）
#[arg(long)]
save_html: bool,
```

## 🎯 功能特點

### 1. **自動儲存**
- ✅ **每次查詢都會自動儲存原始 HTML**
- ✅ **不需要額外的 `--save-html` 參數**
- ✅ **保持向後相容性**

### 2. **檔案命名**
- ✅ **統一的檔案命名格式**: `twse_announcements_YYYYMMDD.html`
- ✅ **與其他檔案格式保持一致**
- ✅ **便於檔案管理和識別**

### 3. **資料完整性**
- ✅ **完整保存原始 HTML 回應**
- ✅ **包含所有隱藏的 input 欄位**
- ✅ **保留完整的表格結構和樣式**

## 📊 資料處理流程

### 完整的資料流程
```
證交所查詢 → 原始 HTML 回應 → 自動儲存 HTML 檔案
                              ↓
                         解析擷取資料
                              ↓
                    ┌─────────────────────┐
                    ├─ MongoDB: 結構化資料 (不含 raw_html)
                    ├─ JSON 檔案: 結構化資料 (不含 raw_html)
                    └─ HTML 檔案: 完整原始資料 (完整備份)
```

### 檔案輸出
```
每次查詢都會產生：
✅ twse_announcements_20250818.html - 原始 HTML (自動儲存)
✅ twse_announcements_20250818.json - 結構化 JSON (可選)
✅ twse_announcements_20250818.txt  - 文字格式 (可選)
```

## 🚀 使用方式

### 1. **基本查詢（自動儲存 HTML）**
```bash
# 只查詢並自動儲存 HTML
./target/release/twse-announcements.exe --date 2025-08-18

# 查詢並儲存到 MongoDB（自動儲存 HTML）
./target/release/twse-announcements.exe --date 2025-08-18 --save-mongodb

# 查詢並輸出 JSON（自動儲存 HTML）
./target/release/twse-announcements.exe --date 2025-08-18 --format json
```

### 2. **完整功能（自動儲存 HTML）**
```bash
# 查詢、儲存 MongoDB、輸出 JSON（自動儲存 HTML）
./target/release/twse-announcements.exe --date 2025-08-18 --format json --save-mongodb
```

### 3. **向後相容（可選參數）**
```bash
# --save-html 參數仍然可用，但已經是預設行為
./target/release/twse-announcements.exe --date 2025-08-18 --save-html
```

## 📁 檔案輸出範例

### 執行結果
```bash
$ ./target/release/twse-announcements.exe --date 2025-08-18 --format json --save-mongodb

查詢日期: 2025-08-18
🔍 發現 input 欄位: name='h07', value='20250708'
📅 擷取事實發生日: h07 = 20250708 -> 2025-07-08
...
✅ 原始 HTML 檔案已自動儲存: twse_announcements_20250818.html
Upsert 操作完成 (不包含原始HTML以減少資料庫大小):
  新增: 176 筆資料
JSON 檔案已儲存: twse_announcements_20250818.json (不包含原始HTML以減少檔案大小)
```

### 檔案大小
```bash
$ ls -lh twse_announcements_20250818.*
-rw-r--r-- 1 User 197121 441K Aug 18 23:18 twse_announcements_20250818.html  # 原始HTML
-rw-r--r-- 1 User 197121 247K Aug 18 23:18 twse_announcements_20250818.json  # 結構化JSON
-rw-r--r-- 1 User 197121  25K Aug 18 19:36 twse_announcements_20250818.txt   # 文字格式
```

## 🔍 原始 HTML 內容

### HTML 檔案包含
- ✅ **完整的 HTTP 回應內容**
- ✅ **所有隱藏的 input 欄位**
- ✅ **事實發生日資料** (`name='h07' value='20250708'`)
- ✅ **條款代號資料** (`name='h06' value='51'`)
- ✅ **詳細內容資料** (`name='h08' value='...'`)
- ✅ **完整的表格結構和樣式**

### HTML 檔案用途
- ✅ **資料備份和恢復**
- ✅ **除錯和問題排查**
- ✅ **深度分析和研究**
- ✅ **法規遵循和稽核**

## 🎯 功能優勢

### 1. **自動化**
- ✅ **無需手動指定參數**
- ✅ **每次查詢都會自動備份**
- ✅ **減少人為疏失**

### 2. **資料安全**
- ✅ **完整的原始資料備份**
- ✅ **多重資料格式保存**
- ✅ **資料恢復能力**

### 3. **使用便利**
- ✅ **簡化命令列參數**
- ✅ **保持向後相容性**
- ✅ **統一的檔案命名**

### 4. **儲存效率**
- ✅ **MongoDB 和 JSON 不含 raw_html（節省空間）**
- ✅ **HTML 檔案完整保存（完整備份）**
- ✅ **最佳的儲存策略**

## 🔄 資料恢復

### 如需重新處理原始資料
```bash
# 使用已儲存的 HTML 檔案重新處理
# （未來可擴展此功能）
cat twse_announcements_20250818.html | grep "name='h07'"
```

### 原始資料分析
```bash
# 檢查事實發生日欄位
grep -o "name='h[0-9]*7' value='[0-9]*'" twse_announcements_20250818.html

# 檢查條款代號欄位
grep -o "name='h[0-9]*6' value='[0-9]*'" twse_announcements_20250818.html
```

## 📊 統計資訊

### 檔案大小比較
- **HTML 檔案**: 441KB (完整原始資料)
- **JSON 檔案**: 247KB (結構化資料，不含 raw_html)
- **TXT 檔案**: 25KB (人類可讀格式)

### 資料完整性
- **HTML 檔案**: 100% 完整原始資料
- **JSON 檔案**: 100% 結構化欄位，不含 raw_html
- **MongoDB**: 100% 結構化欄位，不含 raw_html

## 🎉 實現成果

### 成功實現
- ✅ **自動儲存原始 HTML**: 每次查詢都會自動保存
- ✅ **簡化使用方式**: 不需要額外參數
- ✅ **保持向後相容**: 原有參數仍然可用
- ✅ **資料完整性**: 原始資料完整備份

### 技術優勢
- ✅ **自動化程度高**: 減少人為操作
- ✅ **資料安全性**: 多重備份策略
- ✅ **儲存效率**: 最佳化的儲存方案
- ✅ **使用便利性**: 簡化的命令列介面

### 使用者體驗
- ✅ **操作簡單**: 一個命令完成所有功能
- ✅ **資料安全**: 自動備份原始資料
- ✅ **檔案管理**: 統一的檔案命名規則
- ✅ **功能完整**: 滿足所有使用需求

🎊 **自動儲存原始 HTML 功能已完全實現！現在每次執行查詢重大資訊時都會自動將原始 HTML 存檔，確保資料的完整性和安全性！**
