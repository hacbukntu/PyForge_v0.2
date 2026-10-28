//! Hashes de contraseñas y tokens CSRF vinculados a una sesión opaca del servidor.
use argon2::{
    password_hash::{PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rand::{rngs::OsRng, RngCore};
use subtle::ConstantTimeEq;

/// Genera 256 bits aleatorios; no deriva secretos del reloj ni del nombre del usuario.
pub fn token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Argon2id almacena sal y parámetros en formato PHC. Los límites evitan entradas gigantes.
pub fn hash_password(password: &str) -> Result<String, String> {
    if !(12..=256).contains(&password.len()) {
        return Err("La clave debe tener entre 12 y 256 bytes".into());
    }
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|_| "No se pudo proteger la clave".into())
}

/// Verifica una contraseña sin devolver el hash al consumidor.
pub fn verify_password(password: &str, hash: &str) -> bool {
    if password.len() > 256 {
        return false;
    }
    PasswordHash::new(hash).ok().is_some_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}

/// La longitud fija y comparación constante evitan aceptar prefijos o tokens vacíos.
pub fn valid_csrf(expected: &str, supplied: &str) -> bool {
    expected.len() == 64
        && supplied.len() == 64
        && bool::from(expected.as_bytes().ct_eq(supplied.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_y_tokens() {
        let a = hash_password("Una clave de prueba larga").unwrap();
        let b = hash_password("Una clave de prueba larga").unwrap();
        assert_ne!(a, b);
        assert!(verify_password("Una clave de prueba larga", &a));
        assert!(!verify_password("incorrecta", &a));
        let t = token();
        assert!(valid_csrf(&t, &t));
        assert!(!valid_csrf(&t, ""));
    }
}
