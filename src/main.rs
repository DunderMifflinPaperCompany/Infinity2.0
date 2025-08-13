use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    tera: Arc<Tera>,
}

#[derive(Serialize)]
struct Employee {
    name: String,
    title: String,
    department: String,
    years_service: u8,
    photo: String,
    quote: String,
}

#[derive(Serialize)]
struct NewsItem {
    title: String,
    content: String,
    date: String,
    author: String,
}

async fn home_handler(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let mut context = Context::new();
    
    // Add employee data
    let employees = vec![
        Employee {
            name: "Michael Scott".to_string(),
            title: "Regional Manager".to_string(),
            department: "Management".to_string(),
            years_service: 15,
            photo: "/static/images/michael.jpg".to_string(),
            quote: "That's what she said!".to_string(),
        },
        Employee {
            name: "Jim Halpert".to_string(),
            title: "Sales Representative".to_string(),
            department: "Sales".to_string(),
            years_service: 8,
            photo: "/static/images/jim.jpg".to_string(),
            quote: "Bears. Beets. Battlestar Galactica.".to_string(),
        },
        Employee {
            name: "Dwight K. Schrute".to_string(),
            title: "Assistant Regional Manager".to_string(),
            department: "Sales".to_string(),
            years_service: 10,
            photo: "/static/images/dwight.jpg".to_string(),
            quote: "FACT: Bears eat beets.".to_string(),
        },
        Employee {
            name: "Pam Beesly".to_string(),
            title: "Office Administrator".to_string(),
            department: "Administration".to_string(),
            years_service: 7,
            photo: "/static/images/pam.jpg".to_string(),
            quote: "I'm really happy I'm here.".to_string(),
        },
    ];
    
    // Add news items
    let news = vec![
        NewsItem {
            title: "Infinity 2.0 Launch: Revolutionary Upgrade!".to_string(),
            content: "Our new Infinity 2.0 system promises 400% more efficiency with 73% fewer bugs than the previous version. Features include: Advanced CRM integration, Mobile-first design, AI-powered paper recommendations, and Blockchain-based supply chain tracking.".to_string(),
            date: "2024-01-15".to_string(),
            author: "Ryan Howard".to_string(),
        },
        NewsItem {
            title: "Q4 Sales Records Broken Again!".to_string(),
            content: "Thanks to our innovative sales strategies and the power of Infinity 2.0, Scranton branch has exceeded all expectations. Special recognition goes to our top performers in the field.".to_string(),
            date: "2024-01-10".to_string(),
            author: "Michael Scott".to_string(),
        },
        NewsItem {
            title: "New Mobile App Available".to_string(),
            content: "Download the Dunder Mifflin Infinity 2.0 mobile app for real-time paper ordering, inventory tracking, and exclusive paper deals. Available on BlackBerry and iPhone.".to_string(),
            date: "2024-01-08".to_string(),
            author: "IT Department".to_string(),
        },
    ];
    
    context.insert("employees", &employees);
    context.insert("news", &news);
    context.insert("version", "2.0");
    context.insert("company", "Dunder Mifflin Paper Company");
    
    match state.tera.render("index.html", &context) {
        Ok(rendered) => Ok(Html(rendered)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[tokio::main]
async fn main() {
    // Initialize Tera template engine
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => Arc::new(t),
        Err(e) => {
            println!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    let app_state = AppState { tera };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(home_handler))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Run our app with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Dunder Mifflin Infinity 2.0 server running on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
