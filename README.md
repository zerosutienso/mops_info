# 台灣證交所重大訊息查詢 CLI

這是一個用 Rust 寫的命令列工具，用來查詢台灣證券交易所的重大訊息公告。

## 功能特色

- 🔍 查詢指定日期的重大訊息（預設為當日）
- 🏢 可依公司代號篩選
- 📊 支援多種輸出格式（表格、JSON、HTML、TXT）
- 📋 JSON 格式包含完整詳細資料
- 💾 自動儲存檔案功能
- 🗄️ MongoDB 資料庫整合
- 📈 支援歷史資料分析
- ⚡ 使用 Rust 開發，執行快速

## 安裝

確保你已經安裝了 Rust 開發環境，然後執行：

```bash
cargo build --release
```

## 使用方法

### 基本用法

查詢今日的重大訊息：
```bash
cargo run
```

### 指定日期查詢

查詢特定日期的重大訊息：
```bash
cargo run -- --date 2025-08-15
```

### 篩選特定公司

查詢特定公司的重大訊息：
```bash
cargo run -- --company 2330
```

### 不同輸出格式

表格格式（預設）：
```bash
cargo run -- --format table
```

JSON 格式：
```bash
cargo run -- --format json
```

HTML 格式（原始回應）：
```bash
cargo run -- --format html
```

TXT 格式（純文字檔案）：
```bash
cargo run -- --format txt
```

### 儲存原始 HTML

```bash
cargo run -- --save-html
```

### MongoDB 資料庫整合

儲存到 MongoDB 資料庫：
```bash
# 基本用法
cargo run -- --save-mongodb

# 指定日期和公司
cargo run -- --date 2025-08-15 --company 2330 --save-mongodb

# 自訂 MongoDB 設定
cargo run -- --save-mongodb --mongodb-uri "mongodb://localhost:27017" --mongodb-database "twse_db"
```

### 完整參數範例

```bash
# 完整功能展示
cargo run -- --date 2025-08-15 --company 2330 --format json --save-html --save-mongodb --output my_data
```

## 命令列參數

### 基本參數
- `-d, --date <DATE>`: 查詢日期，格式為 YYYY-MM-DD（預設為今日）
- `-c, --company <CODE>`: 公司代號篩選
- `-f, --format <FORMAT>`: 輸出格式，可選 `table`、`json`、`html`、`txt`（預設為 table）
- `--save-html`: 儲存原始 HTML 回應到檔案
- `-o, --output <PREFIX>`: 輸出檔案前綴名稱（預設為 twse_announcements）

### MongoDB 參數
- `--save-mongodb`: 儲存資料到 MongoDB
- `--mongodb-uri <URI>`: MongoDB 連接字串（預設：mongodb://localhost:27017）
- `--mongodb-database <DB>`: 資料庫名稱（預設：twse_db）
- `--mongodb-collection <COLLECTION>`: 集合名稱（預設：announcements）

## 輸出範例

### 表格格式
```
查詢日期: 2025-08-17
代號       公司名稱                 日期         時間       標題
--------------------------------------------------------------------------------
2330       台積電                   08/17        14:30      董事會決議股利分派
2317       鴻海                     08/17        16:45      重大訊息公告
```

### JSON 格式（包含詳細資料）
```json
[
  {
    "company_code": "2330",
    "company_name": "台積電",
    "title": "本公司代子公司 TSMC Global Ltd. 公告取得固定收益證券",
    "date": "114/08/15",
    "time": "17:40:31",
    "detail_content": "1.證券名稱:\n公司債。\n2.交易日期:114/8/12~114/8/15\n3.董事會通過日期: 不適用\n4.其他核決日期:\n核決層級:不適用。\n民國114年08月15日\n5.交易數量、每單位價格及交易總金額:\n61747YEC5：400,000 單位；每單位US$97.43；總金額US$39.0 佰萬元。\n06051GJS9：300,000 單位；每單位US$97.60；總金額US$29.3 佰萬元。\n6.處分利益（或損失）（取得有價證券者不適用）:\n不適用\n7.與交易標的公司之關係:\n無。\n8.迄目前為止，累積持有本交易證券（含本次交易）之數量、金額、持股\n比例及權利受限情形（如質押情形）:\n61747YEC5：3,590,000 單位；US$349.0 佰萬元；持股比例：不適用；受限情形：無。\n06051GJS9：3,203,950 單位；US$312.0 佰萬元；持股比例：不適用；受限情形：無。\n9.迄目前為止，依「公開發行公司取得或處分資產處理準則」第三條所列之有價證券投\n資（含本次交易）占公司最近期財務報表中總資產及歸屬於母公司業主之權益之比例\n暨最近期財務報表中營運資金數額:\n5.80%；7.65%；NT$436,219 佰萬元。\n10.取得或處分之具體目的:\n固定收益投資。\n11.本次交易表示異議董事之意見:\n不適用\n12.本次交易為關係人交易:\n否\n13.交易相對人及其與公司之關係:\n不適用\n14.監察人承認或審計委員會同意日期:\n不適用\n15.前已就同一件事件發布重大訊息日期: 不適用\n16.其他敘明事項:\n無。"
  }
]
```

### 無資料時
```
查詢日期: 2025-08-17
今日無重大訊息公告
```

### 檔案輸出
程式會自動產生以下檔案：
- `twse_announcements_YYYYMMDD.html` - 原始 HTML 回應（使用 --save-html）
- `twse_announcements_YYYYMMDD.json` - JSON 格式資料
- `twse_announcements_YYYYMMDD.txt` - 純文字格式資料

## 技術說明

- 使用 `reqwest` 進行 HTTP 請求
- 使用 `scraper` 解析 HTML 回應
- 使用 `clap` 處理命令列參數
- 使用 `chrono` 處理日期時間

## 注意事項

- 本工具僅供學習和個人使用
- 請遵守證交所網站的使用條款
- 建議適度使用，避免對伺服器造成過大負擔

## 授權

MIT License
