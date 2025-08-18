use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    tera: Arc<Tera>,
    chat_state: Arc<Mutex<ChatState>>,
    config: ChatConfig,
}

#[derive(Clone)]
struct ChatConfig {
    feature_enabled: bool,
    admin_mode: bool,
}

#[derive(Default)]
struct ChatState {
    sessions: HashMap<Uuid, ChatSession>,
    offices: Vec<Office>,
    salespeople: Vec<Salesperson>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Office {
    id: String,
    name: String,
    location: String,
    available_salespeople: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Salesperson {
    id: String,
    name: String,
    title: String,
    office_id: String,
    is_available: bool,
    quote: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChatSession {
    id: Uuid,
    customer_id: Option<String>,
    office_id: String,
    salesperson_id: Option<String>,
    status: ChatStatus,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    messages: Vec<ChatMessage>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChatMessage {
    id: Uuid,
    sender: String,
    content: String,
    timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
enum ChatStatus {
    Pending,
    Connected,
    Waiting,
    Ended,
    Failed,
}

#[derive(Serialize, Deserialize)]
struct ChatStartRequest {
    office_id: String,
    customer_name: Option<String>,
}

#[derive(Serialize)]
struct ChatStartResponse {
    session_id: Uuid,
    status: String,
    message: String,
    salesperson: Option<Salesperson>,
}

#[derive(Serialize)]
struct ChatEventLog {
    event_type: String,
    session_id: Uuid,
    office_id: String,
    salesperson_id: Option<String>,
    timestamp: DateTime<Utc>,
    details: String,
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
    
    // Add chat-related data if feature is enabled
    let chat_enabled = state.config.feature_enabled;
    let offices = if chat_enabled {
        let chat_state = state.chat_state.lock().unwrap();
        chat_state.offices.clone()
    } else {
        vec![]
    };
    
    context.insert("employees", &employees);
    context.insert("news", &news);
    context.insert("version", "2.0");
    context.insert("company", "Dunder Mifflin Paper Company");
    context.insert("chat_enabled", &chat_enabled);
    context.insert("offices", &offices);
    
    match state.tera.render("index.html", &context) {
        Ok(rendered) => Ok(Html(rendered)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Chat API endpoints
async fn start_chat(
    State(state): State<AppState>,
    Json(request): Json<ChatStartRequest>,
) -> Result<Json<ChatStartResponse>, StatusCode> {
    if !state.config.feature_enabled {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let session_id = Uuid::new_v4();
    let mut chat_state = state.chat_state.lock().unwrap();
    
    // Find office
    let office_name = {
        let office = chat_state.offices.iter()
            .find(|o| o.id == request.office_id)
            .ok_or(StatusCode::BAD_REQUEST)?;
        office.name.clone()
    };
    
    // Find available salesperson
    let available_salesperson = chat_state.salespeople.iter()
        .find(|s| s.office_id == request.office_id && s.is_available)
        .cloned();
    
    let status = if available_salesperson.is_some() {
        ChatStatus::Connected
    } else {
        ChatStatus::Waiting
    };
    
    let session = ChatSession {
        id: session_id,
        customer_id: request.customer_name,
        office_id: request.office_id.clone(),
        salesperson_id: available_salesperson.as_ref().map(|s| s.id.clone()),
        status: status.clone(),
        started_at: Utc::now(),
        ended_at: None,
        messages: vec![],
    };
    
    chat_state.sessions.insert(session_id, session);
    
    // Log event
    log_chat_event(&format!("chat_started"), session_id, &request.office_id, 
                   available_salesperson.as_ref().map(|s| s.id.as_str()), 
                   &format!("Chat started for office: {}", office_name));
    
    let (status_msg, message) = match &status {
        ChatStatus::Connected => ("connected", format!("Connected to {} from {}", 
                                 available_salesperson.as_ref().unwrap().name, office_name)),
        ChatStatus::Waiting => ("waiting", format!("All {} representatives are currently busy. You are in queue.", office_name)),
        _ => ("error", "Unknown status".to_string()),
    };
    
    Ok(Json(ChatStartResponse {
        session_id,
        status: status_msg.to_string(),
        message,
        salesperson: available_salesperson,
    }))
}

async fn get_offices(State(state): State<AppState>) -> Result<Json<Vec<Office>>, StatusCode> {
    if !state.config.feature_enabled {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    let chat_state = state.chat_state.lock().unwrap();
    Ok(Json(chat_state.offices.clone()))
}

async fn end_chat(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<&'static str>, StatusCode> {
    if !state.config.feature_enabled {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    let mut chat_state = state.chat_state.lock().unwrap();
    
    if let Some(session) = chat_state.sessions.get_mut(&session_id) {
        session.status = ChatStatus::Ended;
        session.ended_at = Some(Utc::now());
        
        // Log event
        log_chat_event(&format!("chat_ended"), session_id, &session.office_id, 
                       session.salesperson_id.as_deref(), "Chat ended by user");
        
        Ok(Json("Chat ended successfully"))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Utility function for logging chat events
fn log_chat_event(event_type: &str, session_id: Uuid, office_id: &str, 
                  salesperson_id: Option<&str>, details: &str) {
    let event = ChatEventLog {
        event_type: event_type.to_string(),
        session_id,
        office_id: office_id.to_string(),
        salesperson_id: salesperson_id.map(|s| s.to_string()),
        timestamp: Utc::now(),
        details: details.to_string(),
    };
    
    // In a real application, this would be sent to a logging service
    println!("CHAT_EVENT: {}", serde_json::to_string(&event).unwrap_or_default());
}

// Initialize chat data
fn initialize_chat_data() -> ChatState {
    let offices = vec![
        Office {
            id: "scranton".to_string(),
            name: "Scranton".to_string(),
            location: "Scranton, PA".to_string(),
            available_salespeople: vec!["michael_scott".to_string(), "jim_halpert".to_string(), "dwight_schrute".to_string()],
        },
        Office {
            id: "stamford".to_string(),
            name: "Stamford".to_string(),
            location: "Stamford, CT".to_string(),
            available_salespeople: vec!["karen_filippelli".to_string(), "andy_bernard".to_string()],
        },
        Office {
            id: "utica".to_string(),
            name: "Utica".to_string(),
            location: "Utica, NY".to_string(),
            available_salespeople: vec!["karen_filippelli".to_string()],
        },
        Office {
            id: "nashua".to_string(),
            name: "Nashua".to_string(),
            location: "Nashua, NH".to_string(),
            available_salespeople: vec!["holly_flax".to_string()],
        },
        Office {
            id: "buffalo".to_string(),
            name: "Buffalo".to_string(),
            location: "Buffalo, NY".to_string(),
            available_salespeople: vec!["todd_packer".to_string()],
        },
    ];
    
    let salespeople = vec![
        Salesperson {
            id: "michael_scott".to_string(),
            name: "Michael Scott".to_string(),
            title: "Regional Manager".to_string(),
            office_id: "scranton".to_string(),
            is_available: true,
            quote: "That's what she said!".to_string(),
        },
        Salesperson {
            id: "jim_halpert".to_string(),
            name: "Jim Halpert".to_string(),
            title: "Sales Representative".to_string(),
            office_id: "scranton".to_string(),
            is_available: true,
            quote: "Bears. Beets. Battlestar Galactica.".to_string(),
        },
        Salesperson {
            id: "dwight_schrute".to_string(),
            name: "Dwight K. Schrute".to_string(),
            title: "Assistant Regional Manager".to_string(),
            office_id: "scranton".to_string(),
            is_available: true,
            quote: "FACT: Bears eat beets.".to_string(),
        },
        Salesperson {
            id: "karen_filippelli".to_string(),
            name: "Karen Filippelli".to_string(),
            title: "Sales Representative".to_string(),
            office_id: "stamford".to_string(),
            is_available: false, // Simulate unavailability
            quote: "I'm just trying to do my job.".to_string(),
        },
        Salesperson {
            id: "andy_bernard".to_string(),
            name: "Andy Bernard".to_string(),
            title: "Sales Representative".to_string(),
            office_id: "stamford".to_string(),
            is_available: true,
            quote: "I went to Cornell. Ever heard of it?".to_string(),
        },
        Salesperson {
            id: "holly_flax".to_string(),
            name: "Holly Flax".to_string(),
            title: "HR Representative".to_string(),
            office_id: "nashua".to_string(),
            is_available: true,
            quote: "Why are you the way that you are?".to_string(),
        },
        Salesperson {
            id: "todd_packer".to_string(),
            name: "Todd Packer".to_string(),
            title: "Traveling Salesman".to_string(),
            office_id: "buffalo".to_string(),
            is_available: false, // Simulate unavailability
            quote: "What's up, Halpert?".to_string(),
        },
    ];
    
    ChatState {
        sessions: HashMap::new(),
        offices,
        salespeople,
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

    // Initialize chat state and configuration
    let chat_state = Arc::new(Mutex::new(initialize_chat_data()));
    let config = ChatConfig {
        feature_enabled: true, // Feature flag - can be configured via environment variable
        admin_mode: false,
    };

    let app_state = AppState { 
        tera, 
        chat_state,
        config,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(home_handler))
        .route("/api/chat/start", post(start_chat))
        .route("/api/chat/end/:session_id", post(end_chat))
        .route("/api/offices", get(get_offices))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Run our app with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Dunder Mifflin Infinity 2.0 server running on http://0.0.0.0:3000");
    println!("💬 Chat feature enabled - connecting customers to their favorite salespeople!");
    axum::serve(listener, app).await.unwrap();
}
