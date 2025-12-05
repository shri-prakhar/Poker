use anyhow::Ok;
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{self, PasswordHash, PasswordHasher, SaltString, rand_core::OsRng},
};
use tokio::task;

pub async fn hash_password(password: &str) -> anyhow::Result<String> {
    let pwd = password.to_owned();
    task::spawn_blocking(move || {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2.hash_password(pwd.as_bytes(), &salt)?;
        Ok::<String, anyhow::Error>(hash);
    })
    .await?
}

pub async fn verify_password(hash: &str, candidate: &str) -> bool {
    let hash_owned = hash.to_owned();
    let candidate_owned = candidate.to_owned();
    task::spawn_blocking(move || match PasswordHash::new(&hash_owned) {
        Ok(parsed) => Argon2::default().verify_password(candidate_owned.as_bytes(), &parsed),
        Err(_) => false,
    })
    .await
    .unwrap_or(false)
}
