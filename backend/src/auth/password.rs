use anyhow::Ok;
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::{self, SaltString, rand_core::OsRng , PasswordHasher , PasswordHash}};

pub fn hash_password(password:&str) -> anyhow::Result<String>{
  let argon2 = Argon2::default();
  let salt = SaltString::generate(&mut OsRng);
  let hash = argon2.hash_password(password.as_bytes(), &salt)?;
  Ok(hash);
}

pub fn verify_password(hash:&str , candidate: &str) -> bool {
  match PasswordHash::new(hash){
    Ok(parsed) => Argon2::default().verify_password(candidate.as_bytes(), &parsed),
    Err(_) => false,
  }
}