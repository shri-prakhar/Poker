use argon2::{Argon2, password_hash::{SaltString, rand_core::OsRng, PasswordHasher, PasswordHash, PasswordVerifier}};
use tokio::task;

pub async fn hash_password(password: &str) -> anyhow::Result<String> {
    let pwd = password.to_owned();
     let handle = task::spawn_blocking(move || -> anyhow::Result<String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(pwd.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("failed to hash password: {}", e))?;

        Ok(password_hash.to_string())
    });

    let inner_result = handle
        .await
        .map_err(|e| anyhow::anyhow!("spawn_blocking join error: {}", e))?;

    inner_result
}

pub async fn verify_password(hash: &str, candidate: &str) -> bool {
    let hash_owned = hash.to_owned();
    let candidate_owned = candidate.to_owned();
    task::spawn_blocking(move || {
        match PasswordHash::new(&hash_owned) {
        Ok(parsed) => Argon2::default().verify_password(candidate_owned.as_bytes(), &parsed).is_ok(),
        Err(_) => false,
}})
    .await
    .unwrap_or(false)
}
