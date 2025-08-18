use clap::Parser;
use std::net::SocketAddr;
use tokio;

// 直接包含 web_viewer 模組的內容
include!("../web_viewer.rs");

#[derive(Parser)]
#[command(name = "twse-web-viewer")]
#[command(about = "台灣證交所重大訊息 Web 查看器")]
struct Args {
    /// MongoDB 連接字串
    #[arg(long, default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,
    
    /// MongoDB 資料庫名稱
    #[arg(long, default_value = "twse_db")]
    mongodb_database: String,
    
    /// MongoDB 集合名稱
    #[arg(long, default_value = "announcements")]
    mongodb_collection: String,
    
    /// Web 服務器監聽位址
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    
    /// Web 服務器監聽埠號
    #[arg(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("🚀 啟動台灣證交所重大訊息 Web 查看器");
    println!("📊 MongoDB URI: {}", args.mongodb_uri);
    println!("🗄️  資料庫: {}", args.mongodb_database);
    println!("📁 集合: {}", args.mongodb_collection);
    
    // 測試 MongoDB 連接
    println!("🔗 正在連接 MongoDB...");
    match mongodb::Client::with_uri_str(&args.mongodb_uri).await {
        Ok(client) => {
            // 測試連接
            match client.database(&args.mongodb_database)
                .run_command(mongodb::bson::doc! {"ping": 1}, None).await {
                Ok(_) => println!("✅ MongoDB 連接成功"),
                Err(e) => {
                    eprintln!("❌ MongoDB 連接測試失敗: {}", e);
                    eprintln!("請確認:");
                    eprintln!("1. MongoDB 服務正在運行");
                    eprintln!("2. 連接字串正確: {}", args.mongodb_uri);
                    eprintln!("3. 資料庫存在且有資料");
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 無法連接到 MongoDB: {}", e);
            eprintln!("請確認 MongoDB 服務正在運行，連接字串: {}", args.mongodb_uri);
            return Err(e.into());
        }
    }
    
    // 創建 Web 應用程式
    println!("🌐 正在建立 Web 應用程式...");
    let app = create_app(
        &args.mongodb_uri,
        &args.mongodb_database,
        &args.mongodb_collection,
    ).await?;
    
    // 設定監聽位址
    let addr = SocketAddr::new(
        args.host.parse()?,
        args.port,
    );
    
    println!("🎯 Web 服務器啟動成功！");
    println!("📍 位址: http://{}:{}", args.host, args.port);
    println!("🔗 主頁: http://{}:{}/", args.host, args.port);
    println!("📊 統計: http://{}:{}/stats", args.host, args.port);
    println!("🔌 API: http://{}:{}/api/announcements", args.host, args.port);
    println!();
    println!("💡 使用說明:");
    println!("   - 瀏覽器開啟 http://{}:{} 查看重大訊息", args.host, args.port);
    println!("   - 可以按公司代號、日期篩選");
    println!("   - 支援關鍵字搜尋");
    println!("   - 點擊公告可查看詳細內容");
    println!("   - /stats 頁面提供統計分析");
    println!();
    println!("⏹️  按 Ctrl+C 停止服務器");
    
    // 啟動服務器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
