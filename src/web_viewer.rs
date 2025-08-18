use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use askama::Template;
use bson::{doc, Document};
use chrono::{DateTime, Utc};
use futures_util::stream::StreamExt;
use mongodb::{Client as MongoClient, Collection, options::FindOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};

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

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    announcements: Vec<Announcement>,
    total_count: i64,
    query_params: QueryParams,
    companies: Vec<CompanyInfo>,
    dates: Vec<String>,
}

#[derive(Template)]
#[template(path = "detail.html")]
struct DetailTemplate {
    announcement: Announcement,
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    company_stats: Vec<CompanyStats>,
    date_stats: Vec<DateStats>,
    total_announcements: i64,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub company: Option<String>,
    pub date: Option<String>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            company: None,
            date: None,
            search: None,
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CompanyInfo {
    pub code: String,
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct CompanyStats {
    pub company_code: String,
    pub company_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct DateStats {
    pub date: String,
    pub count: i64,
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
        .route("/detail/:id", get(detail_handler))
        .route("/stats", get(stats_handler))
        .route("/api/announcements", get(api_announcements_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    Ok(app)
}

async fn index_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<Announcement> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    // 建立查詢條件
    let mut filter = doc! {};
    
    if let Some(company) = &params.company {
        if !company.is_empty() {
            filter.insert("company_code", company);
        }
    }
    
    if let Some(date) = &params.date {
        if !date.is_empty() {
            filter.insert("query_date", date);
        }
    }
    
    if let Some(search) = &params.search {
        if !search.is_empty() {
            filter.insert("title", doc! { "$regex": search, "$options": "i" });
        }
    }

    // 計算分頁
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let skip = (page - 1) * limit;

    // 設定查詢選項
    let find_options = FindOptions::builder()
        .skip(skip)
        .limit(limit as i64)
        .build();

    // 查詢資料
    let mut cursor = collection
        .find(filter.clone(), find_options)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut announcements = Vec::new();
    while cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let announcement = cursor.deserialize_current().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        announcements.push(announcement);
    }

    // 計算總數
    let total_count = collection
        .count_documents(filter, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as i64;

    // 獲取公司列表
    let companies = get_companies(&collection).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 獲取日期列表
    let dates = get_dates(&collection).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = IndexTemplate {
        announcements,
        total_count,
        query_params: params,
        companies,
        dates,
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn detail_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<Announcement> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    let object_id = bson::oid::ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let announcement = collection
        .find_one(doc! { "_id": object_id }, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = DetailTemplate { announcement };
    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
}

async fn stats_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let collection: Collection<Document> = state
        .db_client
        .database(&state.database_name)
        .collection(&state.collection_name);

    // 統計各公司公告數量
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
        doc! { "$limit": 20 }
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
                company_stats.push(CompanyStats {
                    company_code: code.to_string(),
                    company_name: name.to_string(),
                    count,
                });
            }
        }
    }

    // 統計各日期公告數量
    let date_pipeline = vec![
        doc! {
            "$group": {
                "_id": "$query_date",
                "count": { "$sum": 1 }
            }
        },
        doc! { "$sort": { "_id": -1 } },
        doc! { "$limit": 30 }
    ];

    let mut date_cursor = collection
        .aggregate(date_pipeline, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut date_stats = Vec::new();
    while date_cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let doc = date_cursor.current();
        if let (Ok(date), Ok(count)) = (doc.get_str("_id"), doc.get_i64("count")) {
            date_stats.push(DateStats {
                date: date.to_string(),
                count,
            });
        }
    }

    // 總公告數量
    let total_announcements = collection
        .count_documents(doc! {}, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? as i64;

    let template = StatsTemplate {
        company_stats,
        date_stats,
        total_announcements,
    };

    Ok(Html(template.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?))
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
    
    if let Some(company) = &params.company {
        if !company.is_empty() {
            filter.insert("company_code", company);
        }
    }
    
    if let Some(date) = &params.date {
        if !date.is_empty() {
            filter.insert("query_date", date);
        }
    }

    let limit = params.limit.unwrap_or(100).min(1000); // 最多 1000 筆

    let find_options = FindOptions::builder()
        .limit(limit as i64)
        .build();

    let mut cursor = collection
        .find(filter, find_options)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut announcements = Vec::new();
    while cursor.advance().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        let announcement = cursor.deserialize_current().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        announcements.push(announcement);
    }

    Ok(axum::Json(announcements))
}

async fn get_companies(collection: &Collection<Announcement>) -> Result<Vec<CompanyInfo>, mongodb::error::Error> {
    let pipeline = vec![
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
        doc! { "$limit": 100 }
    ];

    let mut cursor = collection.aggregate(pipeline, None).await?;
    let mut companies = Vec::new();

    while cursor.advance().await? {
        let doc = cursor.current();
        if let (Ok(id_doc), Ok(count)) = (doc.get_document("_id"), doc.get_i64("count")) {
            if let (Ok(code), Ok(name)) = (id_doc.get_str("company_code"), id_doc.get_str("company_name")) {
                companies.push(CompanyInfo {
                    code: code.to_string(),
                    name: name.to_string(),
                    count,
                });
            }
        }
    }

    Ok(companies)
}

async fn get_dates(collection: &Collection<Announcement>) -> Result<Vec<String>, mongodb::error::Error> {
    let pipeline = vec![
        doc! {
            "$group": {
                "_id": "$query_date",
                "count": { "$sum": 1 }
            }
        },
        doc! { "$sort": { "_id": -1 } },
        doc! { "$limit": 30 }
    ];

    let mut cursor = collection.aggregate(pipeline, None).await?;
    let mut dates = Vec::new();

    while cursor.advance().await? {
        let doc = cursor.current();
        if let Ok(date) = doc.get_str("_id") {
            dates.push(date.to_string());
        }
    }

    Ok(dates)
}
