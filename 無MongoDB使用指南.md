# 無 MongoDB 使用指南

## 🎯 概述

即使沒有安裝 MongoDB，我們的 CLI 工具仍然提供完整的功能！你可以使用所有的查詢和檔案輸出功能。

## ✅ 可用功能

### 1. 🔍 完整查詢功能
- 查詢指定日期的重大訊息
- 篩選特定公司
- 支援所有輸出格式

### 2. 📊 多格式輸出
- **表格格式**: 整齊的控制台顯示
- **JSON 格式**: 包含完整詳細資料
- **TXT 格式**: 純文字檔案
- **HTML 格式**: 原始網頁內容

### 3. 💾 檔案儲存
- 自動儲存到檔案
- 自訂檔案名稱
- 包含日期的檔案命名

## 🚀 使用範例

### 基本查詢
```bash
# 查詢今日重大訊息
cargo run

# 查詢指定日期
cargo run -- --date 2025-08-15

# 查詢特定公司
cargo run -- --date 2025-08-15 --company 2330
```

### JSON 格式（推薦）
```bash
# 台積電詳細資料
cargo run -- --date 2025-08-15 --company 2330 --format json

# 國泰金詳細資料
cargo run -- --date 2025-08-15 --company 2882 --format json

# 當日所有資料
cargo run -- --date 2025-08-15 --format json
```

### TXT 格式
```bash
# 純文字格式，易於閱讀
cargo run -- --date 2025-08-15 --format txt

# 自訂檔案名稱
cargo run -- --date 2025-08-15 --company 2882 --format txt --output cathay_report
```

### HTML 格式
```bash
# 保存原始網頁
cargo run -- --date 2025-08-15 --format html --save-html
```

## 📁 檔案管理

### 自動生成的檔案
```bash
# 執行後會自動生成
cargo run -- --date 2025-08-15 --format json --output my_data

# 生成的檔案
my_data_20250815.json  # JSON 格式資料
```

### 檔案命名規則
- 格式：`{前綴}_{YYYYMMDD}.{副檔名}`
- 範例：
  - `twse_announcements_20250815.json`
  - `cathay_report_20250815.txt`
  - `tsmc_data_20250815.html`

## 📊 實際測試範例

### 範例 1：查詢台積電
```bash
cargo run -- --date 2025-08-15 --company 2330 --format json
```

**輸出**：
```json
[
  {
    "company_code": "2330",
    "company_name": "台積電",
    "title": "本公司代子公司 TSMC Global Ltd. 公告取得固定收益證券",
    "date": "114/08/15",
    "time": "17:40:31",
    "detail_content": "1.證券名稱:\n公司債。\n2.交易日期:114/8/12~114/8/15...",
    "created_at": "2025-08-17T17:35:24.374884700Z"
  }
]
```

### 範例 2：查詢國泰金
```bash
cargo run -- --date 2025-08-15 --company 2882 --format txt
```

**輸出檔案** (`twse_announcements_20250815.txt`)：
```
台灣證交所重大訊息
==================

代號     公司名稱             日期       時間     標題
--------------------------------------------------------------------------------
2882     國泰金              114/08/15  16:51:09 公告本公司董事會通過114年上半年度決算財務報告
2882     國泰金              114/08/15  16:51:40 公告本公司新設永續長
2882     國泰金              114/08/15  16:52:13 公告本公司「風險管理委員會」更名為 「風險管理暨資訊安全委員會」暨成員變動
...
```

### 範例 3：查詢當日所有資料
```bash
cargo run -- --date 2025-08-15 --format json
```

**結果**：400+ 筆重大訊息，包含完整詳細資料

## 🔍 資料分析

### 使用 JSON 檔案進行分析

#### 1. 使用 jq 工具（需另外安裝）
```bash
# 統計各公司公告數量
cat twse_announcements_20250815.json | jq 'group_by(.company_code) | map({company: .[0].company_name, count: length}) | sort_by(.count) | reverse'

# 查詢包含特定關鍵字的公告
cat twse_announcements_20250815.json | jq '.[] | select(.title | contains("財務報告"))'
```

#### 2. 使用 Python 分析
```python
import json
import pandas as pd

# 讀取 JSON 檔案
with open('twse_announcements_20250815.json', 'r', encoding='utf-8') as f:
    data = json.load(f)

# 轉換為 DataFrame
df = pd.DataFrame(data)

# 統計分析
print("各公司公告數量:")
print(df['company_name'].value_counts().head(10))

print("\n包含關鍵字的公告:")
keyword_announcements = df[df['title'].str.contains('財務報告', na=False)]
print(keyword_announcements[['company_name', 'title']])
```

#### 3. 使用 Excel 分析
1. 開啟 Excel
2. 資料 → 取得資料 → 從檔案 → 從 JSON
3. 選擇生成的 JSON 檔案
4. 展開資料進行分析

## 📈 批次處理

### 收集歷史資料
```bash
# Windows 批次檔案 (collect_history.bat)
@echo off
for /L %%i in (1,1,31) do (
    echo 正在處理 2025-08-%%i...
    cargo run -- --date 2025-08-%%i --format json --output history_%%i
)
echo 歷史資料收集完成！
```

### 監控特定公司
```bash
# 監控台積電 (monitor_tsmc.bat)
@echo off
:loop
echo %date% %time% - 查詢台積電最新公告
cargo run -- --company 2330 --format json --output tsmc_latest
timeout /t 3600
goto loop
```

## 🛠️ 進階技巧

### 1. 組合查詢
```bash
# 查詢多家公司（需要分別執行）
cargo run -- --date 2025-08-15 --company 2330 --format json --output tsmc
cargo run -- --date 2025-08-15 --company 2317 --format json --output foxconn
cargo run -- --date 2025-08-15 --company 2882 --format json --output cathay
```

### 2. 資料驗證
```bash
# 檢查是否有資料
cargo run -- --date 2025-08-15 --company 9999
# 如果公司代號不存在，會顯示 "今日無重大訊息公告"
```

### 3. 效能測試
```bash
# 測試大量資料查詢
time cargo run -- --date 2025-08-15 --format json
```

## 💡 實用建議

### 1. 檔案管理
- 建立專門的資料夾儲存輸出檔案
- 使用有意義的檔案前綴名稱
- 定期清理舊檔案

### 2. 資料分析
- JSON 格式最適合程式處理
- TXT 格式最適合人工閱讀
- HTML 格式保留完整原始資料

### 3. 自動化
- 使用批次檔案進行定期查詢
- 結合工作排程器實現自動化
- 建立監控腳本追蹤特定公司

## 🎯 未來升級

當你準備好使用 MongoDB 時：

1. **安裝 MongoDB**：參考 `MongoDB快速安裝指南.md`
2. **測試連接**：執行 `.\mongodb_setup.bat`
3. **開始使用**：加入 `--save-mongodb` 參數

```bash
# 升級到 MongoDB 版本
cargo run -- --date 2025-08-15 --format json --save-mongodb
```

## 🏆 總結

即使沒有 MongoDB，我們的 CLI 工具仍然是一個強大的台灣證交所重大訊息查詢工具：

- ✅ **完整功能**：所有查詢和輸出功能都可正常使用
- ✅ **詳細資料**：JSON 格式包含完整的公告詳細內容
- ✅ **檔案儲存**：自動儲存到多種格式的檔案
- ✅ **資料分析**：可使用各種工具分析 JSON 資料
- ✅ **自動化**：支援批次處理和監控

這已經是一個非常實用的金融資料查詢工具了！🎉
