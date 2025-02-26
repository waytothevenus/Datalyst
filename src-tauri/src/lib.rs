use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use tauri::{State, Builder, Manager};
use argon2::{self, Config};
use jsonwebtoken::{encode, Header, EncodingKey};
use bson::doc;
use chrono::{Utc, Duration};
#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
    email: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
struct AppState {
    client: Client,
    jwt_secret: String,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use tauri::async_runtime::spawn;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone(); // Clone the app handle to move it into the async task
            spawn(async move {
                // Use the async block to perform the MongoDB setup
                match ClientOptions::parse("mongodb://localhost:27017").await {
                    Ok(client_options) => {
                        match Client::with_options(client_options) {
                            Ok(client) => {
                                let jwt_secret = "datalyst_secret_key".to_string(); // Consider using a more secure method for production
                                app_handle.manage(AppState { client, jwt_secret });
                            }
                            Err(e) => {
                                eprintln!("Failed to create MongoDB client: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse MongoDB options: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![sign_up, sign_in, forgot_password])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
async fn sign_up(state: State<'_, AppState>, username: String, password: String, email: String) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");
    let salt = b"random_salt"; // Replace with a securely generated salt
    let hashed_password = argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).map_err(|e| e.to_string())?;
    
    let new_user = User {
        username,
        password: hashed_password,
        email,
    };
    collection.insert_one(new_user).await.map_err(|e| e.to_string())?;
    Ok("User signed up successfully".into())
}
#[tauri::command]
async fn sign_in(state: State<'_, AppState>, username: String, password: String) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");
    let filter = doc! { "username": &username };
    if let Some(user_doc) = collection.find_one(filter).await.map_err(|e| e.to_string())? {
        let user: User = bson::from_document(user_doc).map_err(|e| e.to_string())?;
        if argon2::verify_encoded(&user.password, password.as_bytes()).map_err(|e| e.to_string())? {
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("valid timestamp")
                .timestamp();
            let claims = Claims {
                sub: username,
                exp: expiration as usize,
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_ref())).map_err(|e| e.to_string())?;
            return Ok(token);
        }
    }
    Err("Invalid username or password".into())
}
#[tauri::command]
async fn forgot_password(state: State<'_, AppState>, email: String) -> Result<String, String> {
    // Specify the type of the collection
    let collection: Collection<User> = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email };
    // Use the specified type for the collection
    if let Some(_) = collection.find_one(filter).await.map_err(|e| e.to_string())? {
        // Here you would send an email to the user with a password reset link
        // For simplicity, we'll just return a success message
        return Ok("Password reset email sent".into());
    }
    Err("Email not found".into())
}