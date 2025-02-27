use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use tauri::{State, Builder, Manager};
use argon2::{self, Config};
use jsonwebtoken::{encode, Header, EncodingKey};
use bson::doc;
use chrono::{Utc, Duration};
#[derive(Debug, Serialize, Deserialize)]
struct User {
    first_name: String,
    last_name: String,
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
async fn sign_up(state: State<'_, AppState>, first_name: String, last_name: String, password: String, email: String) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");

    // Check if the user already exists
    let filter = doc! { "email": &email };
    if collection.find_one(filter).await.map_err(|e| e.to_string())?.is_some() {
        return Err("An account with this email already exists".into());
    }

    let salt = b"random_salt"; // Replace with a securely generated salt
    let hashed_password = argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).map_err(|e| e.to_string())?;

    let new_user = User {
        first_name,
        last_name,
        password: hashed_password,
        email,
    };

    collection.insert_one(new_user).await.map_err(|e| e.to_string())?;
    Ok("User signed up successfully".into())
}
#[tauri::command]
async fn sign_in(state: State<'_, AppState>, email: String, password: String) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email };
    if let Some(user_doc) = collection.find_one(filter).await.map_err(|e| e.to_string())? {
        let user: User = bson::from_document(user_doc).map_err(|e| e.to_string())?;
        if argon2::verify_encoded(&user.password, password.as_bytes()).map_err(|e| e.to_string())? {
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .expect("valid timestamp")
                .timestamp();
            let claims = Claims {
                sub: email,
                exp: expiration as usize,
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(state.jwt_secret.as_ref())).map_err(|e| e.to_string())?;
            return Ok(token);
        }
    }
    Err("Invalid email or password".into())
}
#[tauri::command]
async fn forgot_password(state: State<'_, AppState>, email: String, password: String) -> Result<String, String> {
    let collection: Collection<User> = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email };
    if let Some(_) = collection.find_one(filter.clone()).await.map_err(|e| e.to_string())? {
        let salt = b"random_salt"; // Replace with a securely generated salt
        let hashed_password = argon2::hash_encoded(password.as_bytes(), salt, &Config::default()).map_err(|e| e.to_string())?;
        let update = doc! { "$set": { "password": hashed_password } };
        collection.update_one(filter, update).await.map_err(|e| e.to_string())?;
        return Ok("Password reset successfully".into());
    }
    Err("User not found".into())
}