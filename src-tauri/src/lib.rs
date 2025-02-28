use argon2::{self, Config};
use bson::doc;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use mongodb::{options::ClientOptions, Client, Collection};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tauri::{Builder, Manager, State};
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
    smtp_credentials: Credentials,
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
                                let smtp_credentials = Credentials::new(
                                    "kovalvolodya1@gmail.com".to_string(),
                                    "wsbzxbijiepdivjc".to_string(),
                                );
                                app_handle.manage(AppState {
                                    client,
                                    jwt_secret,
                                    smtp_credentials,
                                });
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
        .invoke_handler(tauri::generate_handler![
            sign_up,
            sign_in,
            forgot_password,
            reset_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
async fn sign_up(
    state: State<'_, AppState>,
    first_name: String,
    last_name: String,
    password: String,
    email: String,
) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");

    // Check if the user already exists
    let filter = doc! { "email": &email };
    if collection
        .find_one(filter)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err("An account with this email already exists".into());
    }

    let salt = b"random_salt"; // Replace with a securely generated salt
    let hashed_password = argon2::hash_encoded(password.as_bytes(), salt, &Config::default())
        .map_err(|e| e.to_string())?;

    let new_user = User {
        first_name,
        last_name,
        password: hashed_password,
        email,
    };

    collection
        .insert_one(new_user)
        .await
        .map_err(|e| e.to_string())?;
    Ok("User signed up successfully".into())
}
#[tauri::command]
async fn sign_in(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<String, String> {
    let collection = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email };
    if let Some(user_doc) = collection
        .find_one(filter)
        .await
        .map_err(|e| e.to_string())?
    {
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
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_ref()),
            )
            .map_err(|e| e.to_string())?;
            return Ok(token);
        }
    }
    Err("Invalid email or password".into())
}
#[tauri::command]
async fn forgot_password(state: State<'_, AppState>, email: String) -> Result<String, String> {
    let collection: Collection<User> = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email };
    if let Some(_) = collection
        .find_one(filter.clone())
        .await
        .map_err(|e| e.to_string())?
    {
        let otp = generate_otp(6);
        let update = doc! { "$set": { "otp": &otp } };
        collection
            .update_one(filter, update)
            .await
            .map_err(|e| e.to_string())?;
        if let Err(e) = send_email(&state.smtp_credentials, &email, &otp).await {
            println!("Failed to send email: {}", e);
        } else {
            println!("Email sent successfully");
        };
        return Ok("Password reset email sent".into());
    }
    Err("Email not found".into())
}

#[tauri::command]
async fn reset_password(
    state: State<'_, AppState>,
    email: String,
    otp: String,
    new_password: String,
) -> Result<String, String> {
    let collection: Collection<User> = state.client.database("datalyst").collection("users");
    let filter = doc! { "email": &email, "otp": &otp };
    if let Some(_) = collection
        .find_one(filter.clone())
        .await
        .map_err(|e| e.to_string())?
    {
        let salt = b"random_salt"; // Replace with a securely generated salt
        let hashed_password =
            argon2::hash_encoded(new_password.as_bytes(), salt, &Config::default())
                .map_err(|e| e.to_string())?;
        let update = doc! { "$set": { "password": hashed_password, "otp": null } };
        collection
            .update_one(filter, update)
            .await
            .map_err(|e| e.to_string())?;
        return Ok("Password reset successfully".into());
    }
    Err("Invalid OTP or email".into())
}

fn generate_otp(length: usize) -> String {
    let mut rng = rand::rng();
    let mut otp = String::new();
    for _ in 0..length {
        let mut nums: Vec<i32> = (1..10).collect();
        nums.shuffle(&mut rng);
        let digit = nums.choose(&mut rng).unwrap();
        otp.push_str(&digit.to_string());
    }
    otp
}

async fn send_email(credentials: &Credentials, to: &str, otp: &str) -> Result<(), Box<dyn Error>> {
    let from_email = "kovalvolodya1@gmail.com".parse()?;
    let to_email = to.parse()?;
    let email = Message::builder()
        .from(from_email)
        .to(to_email)
        .subject("Password Reset OTP")
        .body(format!("Your OTP for password reset is: {otp}"))?;

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(credentials.clone())
        .build();
    mailer.send(&email)?;

    Ok(())
}
