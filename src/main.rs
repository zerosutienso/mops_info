use anyhow::Result;
use bson::{doc, Document};
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use mongodb::{Client as MongoClient, Collection};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// pub mod web_viewer; // 暫時禁用，因為模板語法問題

#[derive(Parser)]
#[command(name = "twse-announcements")]
#[command(about = "查詢台灣證交所重大訊息")]
struct Args {
    /// 查詢日期 (格式: YYYY-MM-DD)，預設為今日
    #[arg(short, long)]
    date: Option<String>,
    
    /// 公司代號
    #[arg(short, long)]
    company: Option<String>,
    
    /// 輸出格式 (json, table, html, txt)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// 儲存原始 HTML 回應到檔案（已自動啟用，此參數保留向後相容）
    #[arg(long)]
    save_html: bool,

    /// 輸出檔案前綴名稱
    #[arg(short, long, default_value = "twse_announcements")]
    output: String,

    /// 儲存到 MongoDB
    #[arg(long)]
    save_mongodb: bool,

    /// MongoDB 連接字串
    #[arg(long, default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,

    /// MongoDB 資料庫名稱
    #[arg(long, default_value = "twse_db")]
    mongodb_database: String,

    /// MongoDB 集合名稱
    #[arg(long, default_value = "announcements")]
    mongodb_collection: String,

    /// 防重複模式：upsert(預設), replace, skip
    #[arg(long, default_value = "upsert")]
    duplicate_mode: String,
}

#[derive(Debug, Deserialize)]
struct AnnouncementResponse {
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Announcement {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    company_code: String,
    company_name: String,
    title: String,
    date: String,
    time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    announcement_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fact_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fact_occurrence_date: Option<String>, // 新增事實發生日欄位 (從 h07 擷取)
    #[serde(skip_serializing_if = "Option::is_none")]
    clause_code: Option<String>, // 條款代號欄位
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_html: Option<String>, // 新增原始 HTML 資料欄位
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClauseCode {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    code: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

struct TwseClient {
    client: Client,
}

impl TwseClient {
    fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    async fn fetch_announcements(&self, year: u32, month: u32, day: u32) -> Result<(Vec<Announcement>, String)> {
        let url = "https://mopsov.twse.com.tw/mops/web/ajax_t05st02";

        // 創建字串變數以避免生命週期問題
        let year_str = (year - 1911).to_string(); // 民國年
        let month_str = format!("{:02}", month);
        let day_str = format!("{:02}", day);

        let mut form_data = HashMap::new();
        form_data.insert("encodeURIComponent", "1");
        form_data.insert("step", "1");
        form_data.insert("step00", "0");
        form_data.insert("firstin", "1");
        form_data.insert("off", "1");
        form_data.insert("TYPEK", "all");
        form_data.insert("year", year_str.as_str());
        form_data.insert("month", month_str.as_str());
        form_data.insert("day", day_str.as_str());

        // println!("發送請求到: {}", url);
        // println!("表單資料: {:?}", form_data);

        let response = self.client
            .post(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Origin", "https://mopsov.twse.com.tw")
            .header("Referer", "https://mopsov.twse.com.tw/mops/web/t05st02")
            .form(&form_data)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP request failed: {}", response.status());
        }

        let text = response.text().await?;

        // 解析回應並轉換為結構化資料
        let announcements = self.parse_response(&text)?;
        Ok((announcements, text))
    }

    fn parse_response(&self, html: &str) -> Result<Vec<Announcement>> {
        let document = Html::parse_document(html);
        let mut announcements = Vec::new();

        // 嘗試解析表格資料 - 尋找包含 tblHead 的表格
        let table_selector = Selector::parse("table").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td").unwrap();
        let header_selector = Selector::parse("th.tblHead").unwrap();
        let input_selector = Selector::parse("input[type='hidden']").unwrap();

        for table in document.select(&table_selector) {
            // 檢查是否包含表頭
            if table.select(&header_selector).next().is_some() {
                // 這是資料表格，解析每一行
                for row in table.select(&row_selector) {
                    let cells: Vec<String> = row
                        .select(&cell_selector)
                        .map(|cell| cell.text().collect::<String>().trim().to_string())
                        .collect();

                    // 跳過空行或只有少於5個欄位的行
                    if cells.len() >= 5 && !cells[0].is_empty() {
                        let date = cells[0].trim().to_string();
                        let time = cells[1].trim().to_string();
                        let company_code = cells[2].trim().to_string();
                        let company_name = cells[3].trim().to_string();
                        let title = cells[4].trim().to_string();

                        // 提取詳細資料
                        let (detail_content, announcement_type, fact_date, clause_code, fact_occurrence_date, raw_html) = self.extract_detail_info(&row, &input_selector);

                        // 過濾掉無效的資料
                        if !date.is_empty() && !company_code.is_empty() && !title.is_empty() {
                            // 移除標題中的換行符號，用空格取代
                            let clean_title = title.replace('\n', " ").replace('\r', " ");

                            let announcement = Announcement {
                                id: None,
                                company_code,
                                company_name,
                                title: clean_title,
                                date,
                                time,
                                detail_content,
                                announcement_type,
                                fact_date,
                                fact_occurrence_date,
                                clause_code,
                                raw_html: Some(raw_html),
                                created_at: Some(chrono::Utc::now()),
                                query_date: None, // 將在 main 函數中設定
                            };
                            announcements.push(announcement);
                        }
                    }
                }
                break; // 找到資料表格後就停止
            }
        }

        // 如果沒有找到表格資料，檢查是否有"沒有找到重大訊息"的訊息
        if announcements.is_empty() {
            // 檢查完整的 HTML 內容
            let html_lower = html.to_lowercase();
            if html.contains("沒有找到重大訊息") || html.contains("無重大訊息") ||
               html.contains("沒有找到") || html_lower.contains("no data") {
                println!("今日無重大訊息公告");
            } else {
                println!("未找到表格資料，可能的原因：");
                println!("1. 今日確實無重大訊息");
                println!("2. 網站結構已變更");
                println!("3. 請求參數需要調整");
                println!("\n如需除錯，請檢查原始回應內容。");
            }
        }

        Ok(announcements)
    }

    fn extract_detail_info(&self, row: &scraper::ElementRef, input_selector: &Selector) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String) {
        let mut detail_content = None;
        let mut announcement_type = None;
        let mut fact_date = None;
        let mut clause_code = None;
        let mut fact_occurrence_date = None;

        // 保存原始 HTML
        let raw_html = row.html();

        // 在這一行中尋找隱藏的 input 欄位
        for input in row.select(input_selector) {
            if let Some(name) = input.value().attr("name") {
                if let Some(value) = input.value().attr("value") {
                    println!("🔍 發現 input 欄位: name='{}', value='{}'", name, value);

                    match name {
                        name if name.ends_with("6") => { // h06, h16, h26 等包含條款代號
                            if !value.trim().is_empty() {
                                clause_code = Some(value.trim().to_string());
                                println!("📋 擷取條款代號: {} = {}", name, value);
                            }
                        }
                        name if name.ends_with("7") => { // h07, h17, h27 等包含事實發生日
                            if !value.trim().is_empty() {
                                // 將 YYYYMMDD 格式轉換為 YYYY-MM-DD 格式
                                let formatted_date = format_fact_occurrence_date(value);
                                fact_occurrence_date = Some(formatted_date.clone());
                                println!("📅 擷取事實發生日: {} = {} -> {}", name, value, formatted_date);
                            }
                        }
                        name if name.ends_with("8") => { // h08, h18, h28 等包含詳細內容
                            if !value.trim().is_empty() {
                                detail_content = Some(value.trim().to_string());

                                // 嘗試提取事實發生日（從詳細內容中）
                                if let Some(fact_line) = value.lines().find(|line| line.contains("事實發生日")) {
                                    if let Some(date_part) = fact_line.split('：').nth(1) {
                                        fact_date = Some(date_part.trim().to_string());
                                    }
                                }

                                // 嘗試提取公告類型
                                if let Some(type_line) = value.lines().find(|line| line.contains("符合條款")) {
                                    announcement_type = Some(type_line.trim().to_string());
                                }
                            }
                        }
                        _ => {
                            // 記錄所有其他的 input 欄位以供調試
                            if !value.trim().is_empty() {
                                println!("📝 其他欄位: {} = {}", name, value);
                            }
                        }
                    }
                }
            }
        }

        (detail_content, announcement_type, fact_date, clause_code, fact_occurrence_date, raw_html)
    }

    fn parse_text_content(&self, content: &str) -> Result<Vec<Announcement>> {
        let mut announcements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // 找到資料開始的行（跳過標題行）
        let mut data_started = false;

        for line in lines {
            let line = line.trim();

            // 跳過空行和標題行
            if line.is_empty() || line.contains("公司當日重大訊息之詳細內容") ||
               line.contains("代號") || line.contains("----") {
                if line.contains("----") {
                    data_started = true;
                }
                continue;
            }

            if !data_started {
                continue;
            }

            // 解析資料行
            // 格式看起來是: 日期 時間 公司名稱 標題 代號
            if let Some(announcement) = self.parse_announcement_line(line) {
                announcements.push(announcement);
            }
        }

        Ok(announcements)
    }

    fn parse_announcement_line(&self, line: &str) -> Option<Announcement> {
        // 使用正則表達式或字串分割來解析行
        // 格式: 114/08/17 07:00:03 世界健身-KY 公告本公司名稱... 2762

        // 先嘗試找到日期模式 (114/08/17)
        if let Some(date_end) = line.find(' ') {
            let date = &line[..date_end];
            let rest = &line[date_end..].trim_start();

            // 找到時間模式 (07:00:03)
            if let Some(time_end) = rest.find(' ') {
                let time = &rest[..time_end];
                let rest = &rest[time_end..].trim_start();

                // 找到最後的數字（公司代號）
                if let Some(last_space) = rest.rfind(' ') {
                    let company_code = &rest[last_space..].trim();
                    let content = &rest[..last_space].trim();

                    // 找到公司名稱和標題的分界點
                    // 假設公司名稱不會太長，通常在前20個字元內
                    let mut company_name = String::new();
                    let mut title = String::new();

                    // 簡單的分割邏輯：找到第一個中文標點或特定關鍵字
                    if let Some(split_pos) = content.find("公告") {
                        if split_pos > 0 {
                            company_name = content[..split_pos].trim().to_string();
                            title = content[split_pos..].trim().to_string();
                        } else {
                            // 如果找不到合適的分割點，使用前面的部分作為公司名稱
                            let words: Vec<&str> = content.split_whitespace().collect();
                            if !words.is_empty() {
                                company_name = words[0].to_string();
                                title = words[1..].join(" ");
                            }
                        }
                    } else {
                        // 備用分割邏輯
                        let words: Vec<&str> = content.split_whitespace().collect();
                        if !words.is_empty() {
                            company_name = words[0].to_string();
                            title = words[1..].join(" ");
                        }
                    }

                    return Some(Announcement {
                        id: None,
                        company_code: company_code.to_string(),
                        company_name,
                        title,
                        date: date.to_string(),
                        time: time.to_string(),
                        detail_content: None,
                        announcement_type: None,
                        fact_date: None,
                        fact_occurrence_date: None,
                        clause_code: None,
                        raw_html: None,
                        created_at: Some(chrono::Utc::now()),
                        query_date: None,
                    });
                }
            }
        }

        None
    }
}

fn parse_date(date_str: &str) -> Result<(u32, u32, u32)> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
    Ok((date.year() as u32, date.month(), date.day()))
}

fn get_today() -> (u32, u32, u32) {
    let today = Local::now().date_naive();
    (today.year() as u32, today.month(), today.day())
}

fn print_table(announcements: &[Announcement]) {
    if announcements.is_empty() {
        println!("沒有找到重大訊息");
        return;
    }

    println!("{:<8} {:<20} {:<10} {:<8} {}",
             "代號", "公司名稱", "日期", "時間", "標題");
    println!("{}", "-".repeat(80));

    for announcement in announcements {
        // 移除標題中的換行符號，用空格取代
        let clean_title = announcement.title.replace('\n', " ").replace('\r', " ");
        println!("{:<8} {:<20} {:<10} {:<8} {}",
                 announcement.company_code,
                 announcement.company_name,
                 announcement.date,
                 announcement.time,
                 clean_title);
    }
}

fn save_html(html_content: &str, filename: &str) -> Result<()> {
    fs::write(format!("{}.html", filename), html_content)?;
    println!("✅ 原始 HTML 檔案已自動儲存: {}.html", filename);
    Ok(())
}

fn save_json(announcements: &[Announcement], filename: &str) -> Result<()> {
    // 創建不包含 raw_html 的版本以減少檔案大小
    let announcements_for_json: Vec<_> = announcements.iter().map(|announcement| {
        let mut json_announcement = announcement.clone();
        json_announcement.raw_html = None; // 移除 raw_html 欄位
        json_announcement
    }).collect();

    let json_content = serde_json::to_string_pretty(&announcements_for_json)?;
    fs::write(format!("{}.json", filename), json_content)?;
    println!("JSON 檔案已儲存: {}.json (不包含原始HTML以減少檔案大小)", filename);
    Ok(())
}

fn save_txt(announcements: &[Announcement], filename: &str) -> Result<()> {
    let mut content = String::new();
    content.push_str("台灣證交所重大訊息\n");
    content.push_str("==================\n\n");

    if announcements.is_empty() {
        content.push_str("沒有找到重大訊息\n");
    } else {
        content.push_str(&format!("{:<8} {:<20} {:<10} {:<8} {}\n",
                                 "代號", "公司名稱", "日期", "時間", "標題"));
        content.push_str(&format!("{}\n", "-".repeat(80)));

        for announcement in announcements {
            // 移除標題中的換行符號，用空格取代
            let clean_title = announcement.title.replace('\n', " ").replace('\r', " ");
            content.push_str(&format!("{:<8} {:<20} {:<10} {:<8} {}\n",
                                     announcement.company_code,
                                     announcement.company_name,
                                     announcement.date,
                                     announcement.time,
                                     clean_title));
        }
    }

    fs::write(format!("{}.txt", filename), content)?;
    println!("TXT 檔案已儲存: {}.txt", filename);
    Ok(())
}

async fn save_to_mongodb(
    announcements: &[Announcement],
    mongodb_uri: &str,
    database_name: &str,
    collection_name: &str,
    query_date: &str,
    duplicate_mode: &str
) -> Result<()> {
    // 連接到 MongoDB
    let client = MongoClient::with_uri_str(mongodb_uri).await?;
    let database = client.database(database_name);
    let collection: Collection<Announcement> = database.collection(collection_name);

    if announcements.is_empty() {
        println!("沒有資料需要儲存到 MongoDB");
        return Ok(());
    }

    // 準備要插入的資料，加入查詢日期並移除 raw_html
    let mut docs_to_insert = Vec::new();
    for announcement in announcements {
        let mut doc = announcement.clone();
        doc.query_date = Some(query_date.to_string());
        doc.id = None; // 讓 MongoDB 自動生成 ObjectId
        doc.raw_html = None; // 移除 raw_html 欄位以減少資料庫大小
        docs_to_insert.push(doc);
    }

    // 根據防重複模式處理資料
    match duplicate_mode {
        "upsert" => {
            println!("使用 Upsert 模式：自動更新重複資料...");
            let mut inserted_count = 0;
            let mut updated_count = 0;

            for doc in docs_to_insert {
                // 建立唯一識別條件：公司代號 + 日期 + 時間 + 標題
                let filter = doc! {
                    "company_code": &doc.company_code,
                    "date": &doc.date,
                    "time": &doc.time,
                    "title": &doc.title
                };

                // 使用 replace_one 進行 upsert
                let options = mongodb::options::ReplaceOptions::builder()
                    .upsert(true)
                    .build();

                let result = collection
                    .replace_one(filter, &doc, options)
                    .await?;

                if result.upserted_id.is_some() {
                    inserted_count += 1;
                } else if result.modified_count > 0 {
                    updated_count += 1;
                }
            }

            println!("Upsert 操作完成 (不包含原始HTML以減少資料庫大小):");
            println!("  新增: {} 筆資料", inserted_count);
            println!("  更新: {} 筆資料", updated_count);
            println!("  總計: {} 筆資料", inserted_count + updated_count);
        },

        "replace" => {
            println!("使用 Replace 模式：刪除舊資料後重新插入...");

            // 檢查是否已存在相同日期的資料
            let existing_count = collection
                .count_documents(doc! { "query_date": query_date }, None)
                .await?;

            if existing_count > 0 {
                println!("發現 {} 筆相同日期的資料，刪除後重新插入", existing_count);

                // 刪除相同日期的舊資料
                let delete_result = collection
                    .delete_many(doc! { "query_date": query_date }, None)
                    .await?;
                println!("已刪除 {} 筆舊資料", delete_result.deleted_count);
            }

            // 插入新資料
            let insert_result = collection.insert_many(docs_to_insert, None).await?;
            println!("成功插入 {} 筆新資料 (不包含原始HTML以減少資料庫大小)", insert_result.inserted_ids.len());
        },

        "skip" => {
            println!("使用 Skip 模式：跳過重複資料...");

            let mut inserted_count = 0;
            let mut skipped_count = 0;

            for doc in docs_to_insert {
                // 檢查是否已存在相同的資料
                let filter = doc! {
                    "company_code": &doc.company_code,
                    "date": &doc.date,
                    "time": &doc.time,
                    "title": &doc.title
                };

                let existing = collection.find_one(filter, None).await?;

                if existing.is_none() {
                    // 不存在，插入新資料
                    collection.insert_one(&doc, None).await?;
                    inserted_count += 1;
                } else {
                    // 已存在，跳過
                    skipped_count += 1;
                }
            }

            println!("Skip 操作完成 (不包含原始HTML以減少資料庫大小):");
            println!("  新增: {} 筆資料", inserted_count);
            println!("  跳過: {} 筆重複資料", skipped_count);
            println!("  總計處理: {} 筆資料", inserted_count + skipped_count);
        },

        _ => {
            return Err(anyhow::anyhow!("不支援的防重複模式: {}。支援的模式: upsert, replace, skip", duplicate_mode));
        }
    }
    println!("資料庫: {}, 集合: {}", database_name, collection_name);

    Ok(())
}

async fn setup_mongodb_indexes(
    mongodb_uri: &str,
    database_name: &str,
    collection_name: &str
) -> Result<()> {
    let client = MongoClient::with_uri_str(mongodb_uri).await?;
    let database = client.database(database_name);
    let collection: Collection<Document> = database.collection(collection_name);

    // 建立索引以提升查詢效能
    let indexes = vec![
        mongodb::IndexModel::builder()
            .keys(doc! { "company_code": 1 })
            .build(),
        mongodb::IndexModel::builder()
            .keys(doc! { "query_date": 1 })
            .build(),
        mongodb::IndexModel::builder()
            .keys(doc! { "date": 1, "time": 1 })
            .build(),
        mongodb::IndexModel::builder()
            .keys(doc! { "created_at": 1 })
            .build(),
    ];

    collection.create_indexes(indexes, None).await?;
    println!("MongoDB 索引建立完成");

    Ok(())
}

// 初始化條款代號對照表
async fn initialize_clause_codes(collection: &mongodb::Collection<ClauseCode>) -> Result<()> {
    println!("📋 正在初始化條款代號對照表...");

    let clause_codes = vec![
        ("1", "信用異常或股票交易異動"),
        ("2", "涉訟或主管違法"),
        ("3", "停工、減產、資產處理"),
        ("4", "公司法重大決議"),
        ("5", "重整或破產程序"),
        ("6", "高層人事異動或席次不足"),
        ("7", "更換會計師或承銷商"),
        ("8", "重要主管異動"),
        ("9", "會計年度或政策變更"),
        ("10", "重大契約、合作或新產品量產"),
        ("11", "資本變動或合併收購"),
        ("12", "說明會或未申報資訊發布"),
        ("13", "財務預測差異重大"),
        ("14", "股利政策異動或延遲"),
        ("15", "大額投資計畫"),
        ("16", "增資或債券計畫變動"),
        ("17", "股東會召開通知"),
        ("18", "股東會重要決議"),
        ("19", "舞弊、掏空或主管遭羈押"),
        ("20", "資產交易或衍生損失重大"),
        ("21", "經理人或董事競業行為"),
        ("22", "背書保證達標準"),
        ("23", "資金貸與達標準"),
        ("24", "私募證券交易"),
        ("25", "主要客戶或供應商終止往來"),
        ("26", "災難、罷工、資安等重大事件"),
        ("27", "與銀行協商結果確定"),
        ("28", "關係人或債務人信用異常"),
        ("29", "內控聲明或審查報告"),
        ("30", "財報錯誤或遭保留意見"),
        ("31", "財報提報或自結資訊異動"),
        ("32", "股票集中保管不足"),
        ("33", "股權變動通知"),
        ("34", "董監事遭停止職權"),
        ("35", "公司買回股份"),
        ("36", "減資或面額異動作業"),
        ("37", "上市承諾未履行"),
        ("38", "公開收購申報或通知"),
        ("40", "暫停或恢復交易"),
        ("41", "控股公司持股變動"),
        ("42", "終止上市或改列申請"),
        ("43", "重大捐贈"),
        ("44", "委員會反對或董事會逾越建議"),
        ("45", "增資由特定人認購"),
        ("46", "子公司達終止上市標準或營收為零"),
        ("47", "海外財報與台灣準則差異"),
        ("48", "特定營業細則情事"),
        ("49", "子公司控制力喪失或持股下降"),
        ("50", "子公司海外掛牌相關事項"),
        ("51", "其他重大決策或影響股價事件"),
    ];

    // 檢查是否已經初始化
    let count = collection.count_documents(doc! {}, None).await?;
    if count > 0 {
        println!("📋 條款代號對照表已存在，跳過初始化");
        return Ok(());
    }

    let mut documents = Vec::new();
    for (code, description) in clause_codes {
        documents.push(ClauseCode {
            id: None,
            code: code.to_string(),
            description: description.to_string(),
            created_at: Some(chrono::Utc::now()),
        });
    }

    collection.insert_many(documents, None).await?;
    println!("✅ 條款代號對照表初始化完成，共 {} 筆資料", collection.count_documents(doc! {}, None).await?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let (year, month, day) = if let Some(date_str) = args.date {
        parse_date(&date_str)?
    } else {
        get_today()
    };

    println!("查詢日期: {}-{:02}-{:02}", year, month, day);

    let client = TwseClient::new();
    let (announcements, html_content) = client.fetch_announcements(year, month, day).await?;
    
    let filtered_announcements = if let Some(company_code) = args.company {
        announcements.into_iter()
            .filter(|a| a.company_code.contains(&company_code))
            .collect()
    } else {
        announcements
    };

    // 生成檔案名稱（包含日期）
    let filename = format!("{}_{:04}{:02}{:02}", args.output, year, month, day);
    let query_date_str = format!("{:04}-{:02}-{:02}", year, month, day);

    // 自動儲存原始 HTML（每次查詢都會保存）
    save_html(&html_content, &filename)?;

    // 儲存到 MongoDB（如果要求）
    if args.save_mongodb {
        println!("正在連接 MongoDB...");

        // 建立索引
        if let Err(e) = setup_mongodb_indexes(&args.mongodb_uri, &args.mongodb_database, &args.mongodb_collection).await {
            println!("警告：建立 MongoDB 索引失敗: {}", e);
        }

        // 初始化條款代號對照表
        let client = MongoClient::with_uri_str(&args.mongodb_uri).await?;
        let database = client.database(&args.mongodb_database);
        let clause_collection: Collection<ClauseCode> = database.collection("clause_codes");

        if let Err(e) = initialize_clause_codes(&clause_collection).await {
            println!("警告：初始化條款代號對照表失敗: {}", e);
        }

        // 儲存資料
        save_to_mongodb(
            &filtered_announcements,
            &args.mongodb_uri,
            &args.mongodb_database,
            &args.mongodb_collection,
            &query_date_str,
            &args.duplicate_mode
        ).await?;
    }

    // 根據格式輸出或儲存
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&filtered_announcements)?;
            println!("{}", json_output);
            save_json(&filtered_announcements, &filename)?;
        }
        "table" => {
            print_table(&filtered_announcements);
            save_txt(&filtered_announcements, &filename)?;
        }
        "html" => {
            println!("原始 HTML 內容:");
            println!("{}", html_content);
            save_html(&html_content, &filename)?;
        }
        "txt" => {
            save_txt(&filtered_announcements, &filename)?;
            println!("資料已儲存到 TXT 檔案");
        }
        _ => {
            anyhow::bail!("不支援的輸出格式: {}。支援的格式: json, table, html, txt", args.format);
        }
    }

    Ok(())
}

// 格式化事實發生日 (YYYYMMDD -> YYYY-MM-DD)
fn format_fact_occurrence_date(date_str: &str) -> String {
    if date_str.len() == 8 && date_str.chars().all(|c| c.is_ascii_digit()) {
        // YYYYMMDD 格式
        let year = &date_str[0..4];
        let month = &date_str[4..6];
        let day = &date_str[6..8];
        format!("{}-{}-{}", year, month, day)
    } else {
        // 如果不是標準格式，直接返回原始值
        date_str.to_string()
    }
}
