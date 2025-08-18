use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use bson::doc;
use chrono::{DateTime, Utc};
// use futures_util::stream::StreamExt; // 暫時不需要
use mongodb::{Client as MongoClient, Collection, options::FindOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db_client: MongoClient,
    pub database_name: String,
    pub collection_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Announcement {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub company_code: String,
    pub company_name: String,
    pub title: String,
    pub date: String,
    pub time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub announcement_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_occurrence_date: Option<String>, // 新增事實發生日欄位 (從 h07 擷取)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clause_code: Option<String>, // 條款代號欄位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_html: Option<String>, // 新增原始 HTML 資料欄位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClauseCode {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub code: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub company: Option<String>,
    pub date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub search: Option<String>,
    pub limit: Option<u64>,
}

pub async fn create_app(
    mongodb_uri: &str,
    database_name: &str,
    collection_name: &str,
) -> Result<Router, Box<dyn std::error::Error>> {
    let client = MongoClient::with_uri_str(mongodb_uri).await?;
    
    let state = AppState {
        db_client: client,
        database_name: database_name.to_string(),
        collection_name: collection_name.to_string(),
    };

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/announcements", get(api_announcements_handler))
        .route("/api/clause-codes", get(clause_codes_handler))
        .route("/api/stats", get(stats_handler))
        .route("/api/debug", get(debug_handler))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    Html(r#"
<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>台灣證交所重大訊息查看器</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet">
    <link href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css" rel="stylesheet">
    <style>
        .announcement-card {
            transition: all 0.3s ease;
            border-left: 4px solid #007bff;
            margin-bottom: 1rem;
        }
        .announcement-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 6px 12px rgba(0,0,0,0.15);
        }
        .loading {
            text-align: center;
            padding: 2rem;
        }
        .company-datetime-row {
            border-bottom: 1px solid #e9ecef;
            padding-bottom: 0.5rem;
            display: flex;
            align-items: center;
        }
        .company-header {
            font-size: 1.1rem;
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 0;
        }
        .datetime-info {
            font-size: 0.9rem;
            color: #6c757d;
            white-space: nowrap;
            margin-left: 3ch; /* 3 個字符寬度的間距 */
        }

        /* 響應式設計 */
        @media (max-width: 768px) {
            .company-datetime-row {
                flex-direction: column !important;
                align-items: flex-start !important;
            }
            .datetime-info {
                margin-left: 0;
                margin-top: 0.25rem;
            }
            .company-header {
                font-size: 1rem;
            }
        }

        /* 平板和小桌面 */
        @media (min-width: 769px) and (max-width: 1024px) {
            .datetime-info {
                margin-left: 2ch; /* 平板上稍微縮小間距 */
            }
        }
        .announcement-title {
            font-size: 0.95rem;
            color: #34495e;
            margin-bottom: 0.75rem;
            margin-top: 0.5rem;
            line-height: 1.4;
        }
        .announcement-meta {
            font-size: 0.85rem;
            color: #7f8c8d;
            margin-bottom: 0.5rem;
        }
        .detail-content {
            background-color: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 0.375rem;
            padding: 1rem;
            margin-top: 0.75rem;
            font-size: 0.9rem;
            line-height: 1.5;
            color: #495057;
            display: none;
        }
        .detail-content.show {
            display: block;
            animation: fadeIn 0.3s ease-in;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(-10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .toggle-detail-btn {
            font-size: 0.8rem;
            padding: 0.25rem 0.75rem;
            border-radius: 1rem;
        }
        .stats-summary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border-radius: 0.5rem;
            padding: 1.5rem;
            margin-bottom: 2rem;
            text-align: center;
        }
        .compact-view .announcement-card {
            padding: 0.75rem;
        }
    </style>
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-primary">
        <div class="container">
            <a class="navbar-brand" href="/">
                <i class="fas fa-chart-line me-2"></i>
                台灣證交所重大訊息查看器
            </a>
            <div class="navbar-nav ms-auto">
                <a class="nav-link" href="/api/announcements">
                    <i class="fas fa-code me-1"></i>API
                </a>
                <a class="nav-link" href="/api/stats">
                    <i class="fas fa-chart-bar me-1"></i>統計
                </a>
            </div>
        </div>
    </nav>

    <div class="container mt-4">
        <div class="row">
            <div class="col-12">
                <h2 class="mb-4">
                    <i class="fas fa-search me-2"></i>查詢重大訊息
                </h2>
                
                <!-- 搜尋表單 -->
                <div class="card mb-4">
                    <div class="card-body">
                        <div class="row g-3">
                            <div class="col-md-2">
                                <label class="form-label">公司代號</label>
                                <input type="text" id="companyInput" class="form-control" placeholder="例如: 2330">
                            </div>
                            <div class="col-md-2">
                                <label class="form-label">起始日期</label>
                                <input type="date" id="startDateInput" class="form-control">
                            </div>
                            <div class="col-md-2">
                                <label class="form-label">結束日期</label>
                                <input type="date" id="endDateInput" class="form-control">
                            </div>
                            <div class="col-md-3">
                                <label class="form-label">關鍵字搜尋</label>
                                <input type="text" id="searchInput" class="form-control" placeholder="搜尋標題內容...">
                            </div>
                            <div class="col-md-2">
                                <label class="form-label">筆數限制</label>
                                <select id="limitInput" class="form-select">
                                    <option value="50">50 筆</option>
                                    <option value="100">100 筆</option>
                                    <option value="200">200 筆</option>
                                    <option value="500">500 筆</option>
                                    <option value="1000">1000 筆</option>
                                </select>
                            </div>
                            <div class="col-md-1">
                                <label class="form-label">&nbsp;</label>
                                <button type="button" class="btn btn-primary w-100" onclick="searchAnnouncements()">
                                    <i class="fas fa-search me-1"></i>搜尋
                                </button>
                            </div>
                        </div>
                        <div class="row g-2 mt-2">
                            <div class="col-md-12">
                                <div class="btn-group w-100" role="group">
                                    <button type="button" class="btn btn-outline-secondary btn-sm" onclick="setDateRange('today')">
                                        <i class="fas fa-calendar-day me-1"></i>今日
                                    </button>
                                    <button type="button" class="btn btn-outline-secondary btn-sm" onclick="setDateRange('yesterday')">
                                        <i class="fas fa-calendar-minus me-1"></i>昨日
                                    </button>
                                    <button type="button" class="btn btn-outline-secondary btn-sm" onclick="setDateRange('week')">
                                        <i class="fas fa-calendar-week me-1"></i>本週
                                    </button>
                                    <button type="button" class="btn btn-outline-secondary btn-sm" onclick="setDateRange('month')">
                                        <i class="fas fa-calendar-alt me-1"></i>本月
                                    </button>
                                    <button type="button" class="btn btn-outline-secondary btn-sm" onclick="clearDateRange()">
                                        <i class="fas fa-times me-1"></i>清除日期
                                    </button>
                                    <button type="button" class="btn btn-outline-info btn-sm" onclick="resetForm()">
                                        <i class="fas fa-undo me-1"></i>重置
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- 結果區域 -->
                <div id="results">
                    <div class="loading">
                        <i class="fas fa-spinner fa-spin fa-2x"></i>
                        <p class="mt-2">載入中...</p>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/js/bootstrap.bundle.min.js"></script>
    <script>
        // 全域變數
        let clauseCodes = {}; // 條款代號對照表

        // 載入條款代號對照表
        async function loadClauseCodes() {
            try {
                const response = await fetch('/api/clause-codes');
                const codes = await response.json();

                // 轉換為 key-value 對照表
                clauseCodes = {};
                codes.forEach(code => {
                    clauseCodes[code.code] = code.description;
                });

                console.log('📋 條款代號對照表載入完成:', Object.keys(clauseCodes).length, '筆');
            } catch (error) {
                console.error('載入條款代號對照表失敗:', error);
            }
        }

        // 取得條款代號說明
        function getClauseDescription(code) {
            return clauseCodes[code] || '';
        }

        // 頁面載入時初始化
        window.addEventListener('load', function() {
            loadClauseCodes();
        });

        async function searchAnnouncements() {
            const company = document.getElementById('companyInput').value;
            const startDate = document.getElementById('startDateInput').value;
            const endDate = document.getElementById('endDateInput').value;
            const search = document.getElementById('searchInput').value;
            const limit = document.getElementById('limitInput').value;

            const params = new URLSearchParams();
            if (company) params.append('company', company);
            if (startDate) params.append('start_date', startDate);
            if (endDate) params.append('end_date', endDate);
            if (search) params.append('search', search);
            params.append('limit', limit || '50');

            document.getElementById('results').innerHTML = `
                <div class="loading">
                    <i class="fas fa-spinner fa-spin fa-2x"></i>
                    <p class="mt-2">搜尋中...</p>
                </div>
            `;

            try {
                const response = await fetch(`/api/announcements?${params}`);
                const announcements = await response.json();

                displayResults(announcements);
            } catch (error) {
                document.getElementById('results').innerHTML = `
                    <div class="alert alert-danger">
                        <i class="fas fa-exclamation-triangle me-2"></i>
                        載入失敗: ${error.message}
                    </div>
                `;
            }
        }
        
        function displayResults(announcements) {
            if (announcements.length === 0) {
                document.getElementById('results').innerHTML = `
                    <div class="alert alert-info text-center">
                        <i class="fas fa-info-circle me-2"></i>
                        沒有找到符合條件的重大訊息
                    </div>
                `;
                return;
            }

            // 統計資訊
            const companyCount = new Set(announcements.map(a => a.company_code)).size;
            const startDate = document.getElementById('startDateInput').value;
            const endDate = document.getElementById('endDateInput').value;

            let dateRangeText = '';
            if (startDate && endDate) {
                if (startDate === endDate) {
                    dateRangeText = startDate;
                } else {
                    dateRangeText = `${startDate} ~ ${endDate}`;
                }
            } else if (startDate) {
                dateRangeText = `${startDate} 起`;
            } else if (endDate) {
                dateRangeText = `至 ${endDate}`;
            } else {
                dateRangeText = '全部日期';
            }

            let html = `
                <div class="stats-summary">
                    <div class="row text-center">
                        <div class="col-md-3">
                            <h3 class="mb-1">${announcements.length}</h3>
                            <small>重大訊息</small>
                        </div>
                        <div class="col-md-3">
                            <h3 class="mb-1">${companyCount}</h3>
                            <small>家公司</small>
                        </div>
                        <div class="col-md-6">
                            <h3 class="mb-1" style="font-size: 1.2rem;">${dateRangeText}</h3>
                            <small>查詢範圍</small>
                        </div>
                    </div>
                </div>

                <div class="d-flex justify-content-between align-items-center mb-3">
                    <h4 class="mb-0">
                        <i class="fas fa-list me-2"></i>重大訊息列表
                    </h4>
                    <div>
                        <button class="btn btn-outline-secondary btn-sm me-2" onclick="toggleAllDetails()">
                            <i class="fas fa-expand-alt me-1"></i>
                            <span id="toggleAllText">展開全部</span>
                        </button>
                        <button class="btn btn-outline-info btn-sm" onclick="exportData()">
                            <i class="fas fa-download me-1"></i>匯出
                        </button>
                    </div>
                </div>
            `;

            announcements.forEach((announcement, index) => {
                const hasDetail = announcement.detail_content && announcement.detail_content.trim() !== '';
                const detailContent = hasDetail ? announcement.detail_content : '無詳細內容';

                html += `
                    <div class="card announcement-card" data-index="${index}">
                        <div class="card-body">
                            <!-- 公司名稱和時間在同一行 -->
                            <div class="company-datetime-row mb-2">
                                <div class="company-header">
                                    <i class="fas fa-building me-2 text-primary"></i>
                                    ${announcement.company_code} - ${announcement.company_name}
                                </div>
                                <div class="datetime-info">
                                    <small class="text-muted">
                                        <i class="fas fa-clock me-1"></i>
                                        ${announcement.date} ${announcement.time}
                                    </small>
                                </div>
                            </div>

                            <!-- 公告標題 -->
                            <div class="announcement-title">
                                ${announcement.title}
                                ${announcement.clause_code ? `
                                <span class="badge bg-info ms-2" title="${getClauseDescription(announcement.clause_code)}">
                                    <i class="fas fa-gavel me-1"></i>條款 ${announcement.clause_code}
                                </span>
                                ${getClauseDescription(announcement.clause_code) ? `
                                <small class="text-muted ms-2">
                                    <i class="fas fa-info-circle me-1"></i>
                                    ${getClauseDescription(announcement.clause_code)}
                                </small>
                                ` : ''}
                                ` : ''}
                                ${announcement.fact_occurrence_date ? `
                                <span class="badge bg-warning ms-2">
                                    <i class="fas fa-calendar-alt me-1"></i>事實發生日 ${announcement.fact_occurrence_date}
                                </span>
                                ` : ''}
                            </div>

                            <!-- 操作按鈕 -->
                            <div class="d-flex justify-content-between align-items-center">
                                <div>
                                    ${hasDetail ? `
                                    <button class="btn btn-outline-primary toggle-detail-btn"
                                            onclick="toggleDetail(${index})"
                                            id="toggleBtn${index}">
                                        <i class="fas fa-chevron-down me-1"></i>
                                        顯示明細
                                    </button>
                                    ` : `
                                    <span class="text-muted small">
                                        <i class="fas fa-info-circle me-1"></i>
                                        無詳細內容
                                    </span>
                                    `}
                                </div>

                            </div>

                            <!-- 詳細內容區域 -->
                            ${hasDetail ? `
                            <div class="detail-content" id="detail${index}">
                                <div class="d-flex justify-content-between align-items-center mb-2">
                                    <strong class="text-primary">
                                        <i class="fas fa-file-alt me-1"></i>詳細內容
                                    </strong>
                                    <button class="btn btn-sm btn-outline-secondary" onclick="toggleDetail(${index})">
                                        <i class="fas fa-times"></i>
                                    </button>
                                </div>
                                <div style="white-space: pre-wrap; word-wrap: break-word;">
                                    ${detailContent}
                                </div>
                            </div>
                            ` : ''}
                        </div>
                    </div>
                `;
            });

            document.getElementById('results').innerHTML = html;

            // 儲存資料供其他功能使用
            window.currentAnnouncements = announcements;
        }
        
        // 切換單個明細顯示
        function toggleDetail(index) {
            const detailElement = document.getElementById(`detail${index}`);
            const toggleBtn = document.getElementById(`toggleBtn${index}`);

            if (detailElement.classList.contains('show')) {
                detailElement.classList.remove('show');
                toggleBtn.innerHTML = '<i class="fas fa-chevron-down me-1"></i>顯示明細';
                toggleBtn.classList.remove('btn-primary');
                toggleBtn.classList.add('btn-outline-primary');
            } else {
                detailElement.classList.add('show');
                toggleBtn.innerHTML = '<i class="fas fa-chevron-up me-1"></i>隱藏明細';
                toggleBtn.classList.remove('btn-outline-primary');
                toggleBtn.classList.add('btn-primary');
            }
        }

        // 切換全部明細顯示
        function toggleAllDetails() {
            const allDetails = document.querySelectorAll('.detail-content');
            const toggleAllBtn = document.getElementById('toggleAllText');
            const isExpanded = toggleAllBtn.textContent === '收合全部';

            allDetails.forEach((detail, index) => {
                const toggleBtn = document.getElementById(`toggleBtn${index}`);
                if (!toggleBtn) return; // 跳過沒有明細的項目

                if (isExpanded) {
                    // 收合全部
                    detail.classList.remove('show');
                    toggleBtn.innerHTML = '<i class="fas fa-chevron-down me-1"></i>顯示明細';
                    toggleBtn.classList.remove('btn-primary');
                    toggleBtn.classList.add('btn-outline-primary');
                } else {
                    // 展開全部
                    detail.classList.add('show');
                    toggleBtn.innerHTML = '<i class="fas fa-chevron-up me-1"></i>隱藏明細';
                    toggleBtn.classList.remove('btn-outline-primary');
                    toggleBtn.classList.add('btn-primary');
                }
            });

            toggleAllBtn.textContent = isExpanded ? '展開全部' : '收合全部';
        }



        // 匯出資料
        function exportData() {
            if (!window.currentAnnouncements || window.currentAnnouncements.length === 0) {
                alert('沒有資料可以匯出');
                return;
            }

            const csvContent = generateCSV(window.currentAnnouncements);
            const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
            const link = document.createElement('a');
            const url = URL.createObjectURL(blob);
            link.setAttribute('href', url);
            link.setAttribute('download', `重大訊息_${new Date().toISOString().split('T')[0]}.csv`);
            link.style.visibility = 'hidden';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }

        // 生成 CSV 內容
        function generateCSV(announcements) {
            const headers = ['公司代號', '公司名稱', '標題', '日期', '時間', '條款代號', '事實發生日', '詳細內容'];
            const csvRows = [headers.join(',')];

            announcements.forEach(announcement => {
                const row = [
                    `"${announcement.company_code}"`,
                    `"${announcement.company_name}"`,
                    `"${announcement.title.replace(/"/g, '""')}"`,
                    `"${announcement.date}"`,
                    `"${announcement.time}"`,
                    `"${announcement.clause_code || ''}"`,
                    `"${announcement.fact_occurrence_date || ''}"`,
                    `"${(announcement.detail_content || '').replace(/"/g, '""')}"`
                ];
                csvRows.push(row.join(','));
            });

            return '\uFEFF' + csvRows.join('\n'); // 添加 BOM 以支援中文
        }

        // 設定日期範圍
        function setDateRange(range) {
            const today = new Date();
            const startDateInput = document.getElementById('startDateInput');
            const endDateInput = document.getElementById('endDateInput');

            let startDate, endDate;

            switch (range) {
                case 'today':
                    startDate = endDate = today;
                    break;
                case 'yesterday':
                    const yesterday = new Date(today);
                    yesterday.setDate(today.getDate() - 1);
                    startDate = endDate = yesterday;
                    break;
                case 'week':
                    const weekStart = new Date(today);
                    // 計算本週一的日期 (週一為一週開始)
                    const dayOfWeek = today.getDay(); // 0=週日, 1=週一, ..., 6=週六
                    const daysFromMonday = dayOfWeek === 0 ? 6 : dayOfWeek - 1; // 週日時往前6天到週一
                    weekStart.setDate(today.getDate() - daysFromMonday);
                    startDate = weekStart;
                    endDate = today;
                    break;
                case 'month':
                    const monthStart = new Date(today.getFullYear(), today.getMonth(), 1);
                    startDate = monthStart;
                    endDate = today;
                    break;
            }

            startDateInput.value = formatDate(startDate);
            endDateInput.value = formatDate(endDate);
        }

        // 清除日期範圍
        function clearDateRange() {
            document.getElementById('startDateInput').value = '';
            document.getElementById('endDateInput').value = '';
        }

        // 重置表單
        function resetForm() {
            document.getElementById('companyInput').value = '';
            document.getElementById('startDateInput').value = '';
            document.getElementById('endDateInput').value = '';
            document.getElementById('searchInput').value = '';
            document.getElementById('limitInput').value = '50';
        }

        // 格式化日期為 YYYY-MM-DD
        function formatDate(date) {
            const year = date.getFullYear();
            const month = String(date.getMonth() + 1).padStart(2, '0');
            const day = String(date.getDate()).padStart(2, '0');
            return `${year}-${month}-${day}`;
        }

        // 頁面載入時自動搜尋
        window.onload = function() {
            // 設定預設日期為今日
            setDateRange('today');
            searchAnnouncements();
        };
    </script>
</body>
</html>
    "#)
}

async fn api_announcements_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<Announcement> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    let mut filter = doc! {};

    // 公司代號篩選
    if let Some(company) = &params.company {
        if !company.is_empty() {
            filter.insert("company_code", company);
        }
    }

    // 日期範圍篩選 - 使用全面的多格式策略
    if let (Some(start_date), Some(end_date)) = (&params.start_date, &params.end_date) {
        if !start_date.is_empty() && !end_date.is_empty() {
            println!("🔍 查詢日期範圍: {} 到 {}", start_date, end_date);

            // 生成所有可能的日期格式
            let date_conditions = generate_date_conditions(start_date, end_date);

            if !date_conditions.is_empty() {
                println!("📊 使用 {} 個日期查詢條件", date_conditions.len());
                filter.insert("$or", date_conditions);
            } else {
                // 備用查詢：如果所有格式都失敗，使用寬鬆的字串匹配
                println!("⚠️ 使用備用查詢策略");
                filter.insert("$or", vec![
                    doc! { "query_date": doc! { "$gte": start_date, "$lte": end_date } },
                    doc! { "date": doc! { "$gte": start_date, "$lte": end_date } },
                    doc! { "fact_date": doc! { "$gte": start_date, "$lte": end_date } }
                ]);
            }
        }
    } else if let Some(start_date) = &params.start_date {
        if !start_date.is_empty() {
            println!("查詢起始日期: {}", start_date);

            // 使用相同的全面策略
            let date_conditions = generate_single_date_conditions(start_date, true);
            if !date_conditions.is_empty() {
                filter.insert("$or", date_conditions);
            }
        }
    } else if let Some(end_date) = &params.end_date {
        if !end_date.is_empty() {
            println!("查詢結束日期: {}", end_date);

            // 使用相同的全面策略
            let date_conditions = generate_single_date_conditions(end_date, false);
            if !date_conditions.is_empty() {
                filter.insert("$or", date_conditions);
            }
        }
    } else if let Some(date) = &params.date {
        // 保持向後相容性
        if !date.is_empty() {
            println!("查詢單一日期: {}", date);
            filter.insert("query_date", date);
        }
    }

    // 關鍵字搜尋
    if let Some(search) = &params.search {
        if !search.is_empty() {
            filter.insert("title", doc! { "$regex": search, "$options": "i" });
        }
    }

    let limit = params.limit.unwrap_or(50).min(1000); // 最多 1000 筆

    // 多層排序：先按日期，再按時間，最後按建立時間
    let find_options = FindOptions::builder()
        .limit(limit as i64)
        .sort(doc! {
            "date": -1,           // 日期降序 (最新的在前)
            "time": -1,           // 時間降序 (最晚的在前)
            "created_at": -1      // 建立時間降序 (最新的在前)
        })
        .build();

    let mut cursor = collection
        .find(filter, find_options)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut announcements = Vec::new();
    let mut total_count = 0;
    let mut filtered_count = 0;

    while cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let announcement = cursor.deserialize_current().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        total_count += 1;

        // 如果有日期範圍查詢，驗證結果是否真的在範圍內
        if let (Some(start_date), Some(end_date)) = (&params.start_date, &params.end_date) {
            if !start_date.is_empty() && !end_date.is_empty() {
                if is_date_in_range(&announcement, start_date, end_date) {
                    announcements.push(announcement);
                    filtered_count += 1;
                } else {
                    println!("⚠️ 過濾掉範圍外的資料: {} - {}",
                        announcement.company_code,
                        &announcement.date
                    );
                }
            } else {
                announcements.push(announcement);
                filtered_count += 1;
            }
        } else {
            announcements.push(announcement);
            filtered_count += 1;
        }
    }

    if let (Some(start_date), Some(end_date)) = (&params.start_date, &params.end_date) {
        if !start_date.is_empty() && !end_date.is_empty() {
            println!("📊 查詢結果統計: 總共 {} 筆，範圍內 {} 筆", total_count, filtered_count);
        }
    }

    Ok(Json(announcements))
}

// 轉換西元日期到民國年格式
fn convert_to_roc_date(date_str: &str) -> Option<String> {
    // 解析 YYYY-MM-DD 格式
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        println!("日期格式錯誤: {}", date_str);
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month = parts[1];
    let day = parts[2];

    // 轉換為民國年 (西元年 - 1911)
    let roc_year = year - 1911;
    if roc_year <= 0 {
        println!("民國年轉換錯誤: {} -> {}", year, roc_year);
        return None;
    }

    // 格式化為 YYY/MM/DD，確保月日為兩位數
    let formatted = format!("{}/{:02}/{:02}",
        roc_year,
        month.parse::<u32>().unwrap_or(0),
        day.parse::<u32>().unwrap_or(0)
    );

    println!("日期轉換: {} -> {}", date_str, formatted);
    Some(formatted)
}

// 驗證公告日期是否在指定範圍內
fn is_date_in_range(announcement: &Announcement, start_date: &str, end_date: &str) -> bool {
    // 檢查多個可能的日期欄位
    let date_fields = [
        Some(announcement.date.as_str()),
        announcement.query_date.as_ref().map(|s| s.as_str()),
        announcement.fact_date.as_ref().map(|s| s.as_str()),
    ];

    for date_field in date_fields.iter().flatten() {
        if is_single_date_in_range(date_field, start_date, end_date) {
            return true;
        }
    }

    false
}

// 檢查單一日期是否在範圍內
fn is_single_date_in_range(date_str: &str, start_date: &str, end_date: &str) -> bool {
    // 嘗試將各種格式的日期轉換為可比較的格式
    let normalized_date = normalize_date_for_comparison(date_str);
    let normalized_start = normalize_date_for_comparison(start_date);
    let normalized_end = normalize_date_for_comparison(end_date);

    if let (Some(date), Some(start), Some(end)) = (normalized_date, normalized_start, normalized_end) {
        date >= start && date <= end
    } else {
        // 如果無法正規化，使用字串比較作為備用
        date_str >= start_date && date_str <= end_date
    }
}

// 將日期正規化為 YYYYMMDD 格式以便比較
fn normalize_date_for_comparison(date_str: &str) -> Option<String> {
    // 處理 YYYY-MM-DD 格式
    if date_str.contains('-') && date_str.len() == 10 {
        return Some(date_str.replace("-", ""));
    }

    // 處理 YYY/MM/DD 格式 (民國年)
    if date_str.contains('/') {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>()
            ) {
                // 如果是民國年 (3位數)，轉換為西元年
                let western_year = if year < 1000 { year + 1911 } else { year };
                return Some(format!("{:04}{:02}{:02}", western_year, month, day));
            }
        }
    }

    None
}

// 生成全面的日期查詢條件
fn generate_date_conditions(start_date: &str, end_date: &str) -> Vec<bson::Document> {
    let mut conditions = Vec::new();

    // 1. 西元年格式 (YYYY-MM-DD)
    conditions.push(doc! { "query_date": doc! { "$gte": start_date, "$lte": end_date } });
    conditions.push(doc! { "date": doc! { "$gte": start_date, "$lte": end_date } });
    conditions.push(doc! { "fact_date": doc! { "$gte": start_date, "$lte": end_date } });

    // 2. 民國年格式 (YYY/MM/DD)
    if let (Some(start_roc), Some(end_roc)) = (convert_to_roc_date(start_date), convert_to_roc_date(end_date)) {
        conditions.push(doc! { "query_date": doc! { "$gte": start_roc.clone(), "$lte": end_roc.clone() } });
        conditions.push(doc! { "date": doc! { "$gte": start_roc.clone(), "$lte": end_roc.clone() } });
        conditions.push(doc! { "fact_date": doc! { "$gte": start_roc.clone(), "$lte": end_roc.clone() } });

        // 3. 民國年格式變體 (YYY/M/D - 單位數月日)
        if let (Some(start_roc_short), Some(end_roc_short)) = (
            convert_to_roc_date_short(start_date),
            convert_to_roc_date_short(end_date)
        ) {
            conditions.push(doc! { "query_date": doc! { "$gte": start_roc_short.clone(), "$lte": end_roc_short.clone() } });
            conditions.push(doc! { "date": doc! { "$gte": start_roc_short.clone(), "$lte": end_roc_short.clone() } });
            conditions.push(doc! { "fact_date": doc! { "$gte": start_roc_short.clone(), "$lte": end_roc_short.clone() } });
        }
    }

    // 4. 使用正則表達式進行模糊匹配
    let start_parts: Vec<&str> = start_date.split('-').collect();
    let end_parts: Vec<&str> = end_date.split('-').collect();

    if start_parts.len() == 3 && end_parts.len() == 3 {
        let start_year = start_parts[0].parse::<i32>().unwrap_or(0);
        let end_year = end_parts[0].parse::<i32>().unwrap_or(0);
        let start_month = start_parts[1].parse::<i32>().unwrap_or(0);
        let _end_month = end_parts[1].parse::<i32>().unwrap_or(0);
        let start_day = start_parts[2].parse::<i32>().unwrap_or(0);
        let _end_day = end_parts[2].parse::<i32>().unwrap_or(0);

        // 民國年範圍
        let start_roc_year = start_year - 1911;
        let end_roc_year = end_year - 1911;

        if start_roc_year > 0 && end_roc_year > 0 {
            // 使用正則表達式匹配日期範圍
            let date_regex = if start_date == end_date {
                // 單一日期的多種格式
                format!(r"^({}|{})/(0?{}|{})/(0?{}|{})$",
                    start_roc_year, start_year,
                    start_month, start_month,
                    start_day, start_day
                )
            } else {
                // 日期範圍的正則表達式 (簡化版)
                format!(r"^({}|{})/(0?[1-9]|1[0-2])/(0?[1-9]|[12][0-9]|3[01])$",
                    start_roc_year, start_year
                )
            };

            conditions.push(doc! { "query_date": doc! { "$regex": date_regex.clone(), "$options": "i" } });
            conditions.push(doc! { "date": doc! { "$regex": date_regex.clone(), "$options": "i" } });
            conditions.push(doc! { "fact_date": doc! { "$regex": date_regex, "$options": "i" } });
        }
    }

    println!("生成了 {} 個日期查詢條件", conditions.len());
    conditions
}

// 轉換為民國年格式 (單位數月日)
fn convert_to_roc_date_short(date_str: &str) -> Option<String> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    let roc_year = year - 1911;
    if roc_year <= 0 {
        return None;
    }

    // 不補零的格式
    Some(format!("{}/{}/{}", roc_year, month, day))
}

// 生成單一日期查詢條件
fn generate_single_date_conditions(date_str: &str, is_start: bool) -> Vec<bson::Document> {
    let mut conditions = Vec::new();
    let operator = if is_start { "$gte" } else { "$lte" };

    // 1. 西元年格式
    conditions.push(doc! { "query_date": doc! { operator: date_str } });
    conditions.push(doc! { "date": doc! { operator: date_str } });
    conditions.push(doc! { "fact_date": doc! { operator: date_str } });

    // 2. 民國年格式
    if let Some(roc_date) = convert_to_roc_date(date_str) {
        conditions.push(doc! { "query_date": doc! { operator: roc_date.clone() } });
        conditions.push(doc! { "date": doc! { operator: roc_date.clone() } });
        conditions.push(doc! { "fact_date": doc! { operator: roc_date } });
    }

    // 3. 民國年格式 (單位數)
    if let Some(roc_date_short) = convert_to_roc_date_short(date_str) {
        conditions.push(doc! { "query_date": doc! { operator: roc_date_short.clone() } });
        conditions.push(doc! { "date": doc! { operator: roc_date_short.clone() } });
        conditions.push(doc! { "fact_date": doc! { operator: roc_date_short } });
    }

    // 4. 正則表達式匹配
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() == 3 {
        let year = parts[0].parse::<i32>().unwrap_or(0);
        let month = parts[1].parse::<i32>().unwrap_or(0);
        let day = parts[2].parse::<i32>().unwrap_or(0);
        let roc_year = year - 1911;

        if roc_year > 0 {
            // 匹配該日期的多種格式
            let date_regex = format!(r"^({}|{})/(0?{}|{})/(0?{}|{})$",
                roc_year, year, month, month, day, day
            );

            conditions.push(doc! { "query_date": doc! { "$regex": date_regex.clone(), "$options": "i" } });
            conditions.push(doc! { "date": doc! { "$regex": date_regex.clone(), "$options": "i" } });
            conditions.push(doc! { "fact_date": doc! { "$regex": date_regex, "$options": "i" } });
        }
    }

    println!("為 {} 日期生成了 {} 個查詢條件", date_str, conditions.len());
    conditions
}

// 調試處理函數 - 檢查資料庫中的實際資料格式
async fn debug_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<Announcement> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    // 取得前 5 筆資料來檢查格式
    let find_options = FindOptions::builder()
        .limit(5)
        .build();

    let mut cursor = collection
        .find(doc! {}, find_options)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut debug_info = Vec::new();
    while cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let announcement = cursor.deserialize_current().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let info = serde_json::json!({
            "company_code": announcement.company_code,
            "company_name": announcement.company_name,
            "date": announcement.date,
            "time": announcement.time,
            "query_date": announcement.query_date,
            "fact_date": announcement.fact_date,
            "created_at": announcement.created_at,
            "title": announcement.title.chars().take(50).collect::<String>() + "..."
        });
        debug_info.push(info);
    }

    // 統計不同日期格式的資料數量
    let date_stats = analyze_date_formats(&collection).await;

    Ok(Json(serde_json::json!({
        "debug_info": debug_info,
        "total_count": collection.count_documents(doc! {}, None).await.unwrap_or(0),
        "date_format_stats": date_stats
    })))
}

// 分析資料庫中的日期格式分佈
async fn analyze_date_formats(collection: &Collection<Announcement>) -> serde_json::Value {
    let mut stats = std::collections::HashMap::new();

    // 取樣分析前 100 筆資料
    let find_options = FindOptions::builder().limit(100).build();

    if let Ok(mut cursor) = collection.find(doc! {}, find_options).await {
        while let Ok(true) = cursor.advance().await {
            if let Ok(announcement) = cursor.deserialize_current() {
                // 分析 query_date 格式
                if let Some(query_date) = &announcement.query_date {
                    let format_type = detect_date_format(query_date);
                    *stats.entry(format!("query_date_{}", format_type)).or_insert(0) += 1;
                }

                // 分析 date 格式
                let format_type = detect_date_format(&announcement.date);
                *stats.entry(format!("date_{}", format_type)).or_insert(0) += 1;

                // 分析 fact_date 格式
                if let Some(fact_date) = &announcement.fact_date {
                    let format_type = detect_date_format(fact_date);
                    *stats.entry(format!("fact_date_{}", format_type)).or_insert(0) += 1;
                }
            }
        }
    }

    serde_json::json!(stats)
}

// 檢測日期格式類型
fn detect_date_format(date_str: &str) -> String {
    if date_str.contains('-') {
        if date_str.len() == 10 && date_str.matches('-').count() == 2 {
            "YYYY-MM-DD".to_string()
        } else {
            "other_dash".to_string()
        }
    } else if date_str.contains('/') {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() == 3 {
            let year_len = parts[0].len();
            if year_len == 3 {
                "YYY/MM/DD_roc".to_string()
            } else if year_len == 4 {
                "YYYY/MM/DD".to_string()
            } else {
                "other_slash".to_string()
            }
        } else {
            "invalid_slash".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

async fn clause_codes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<ClauseCode> = state
        .db_client
        .database(&state.database_name)
        .collection("clause_codes");

    let mut cursor = collection.find(doc! {}, None).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut clause_codes = Vec::new();
    while cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let clause_code = cursor.deserialize_current().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        clause_codes.push(clause_code);
    }

    // 按條款代號排序
    clause_codes.sort_by(|a, b| {
        let a_num: i32 = a.code.parse().unwrap_or(999);
        let b_num: i32 = b.code.parse().unwrap_or(999);
        a_num.cmp(&b_num)
    });

    Ok(Json(clause_codes))
}

async fn stats_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<bson::Document> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    // 統計總數
    let total_count = collection
        .count_documents(doc! {}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 統計各公司數量
    let company_pipeline = vec![
        doc! {
            "$group": {
                "_id": {
                    "company_code": "$company_code",
                    "company_name": "$company_name"
                },
                "count": { "$sum": 1 }
            }
        },
        doc! { "$sort": { "count": -1 } },
        doc! { "$limit": 10 }
    ];

    let mut company_cursor = collection
        .aggregate(company_pipeline, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut company_stats = Vec::new();
    while company_cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let doc = company_cursor.current();
        if let (Ok(id_doc), Ok(count)) = (doc.get_document("_id"), doc.get_i64("count")) {
            if let (Ok(code), Ok(name)) = (id_doc.get_str("company_code"), id_doc.get_str("company_name")) {
                company_stats.push(serde_json::json!({
                    "company_code": code,
                    "company_name": name,
                    "count": count
                }));
            }
        }
    }

    let stats = serde_json::json!({
        "total_announcements": total_count,
        "top_companies": company_stats
    });

    Ok(Json(stats))
}
