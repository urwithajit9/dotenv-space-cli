/// Restore command - decrypt and restore from backup
///
/// Decrypts AES-256-GCM encrypted backups created by the backup command
use anyhow::{anyhow, Context, Result};
use colored::*;
use dialoguer::{Confirm, Password};
use std::fs;

#[cfg(feature = "backup")]
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
#[cfg(feature = "backup")]
use argon2::{Argon2, PasswordHasher};
#[cfg(feature = "backup")]
use base64::{engine::general_purpose, Engine as _};

pub fn run(backup: String, output: String, verbose: bool) -> Result<()> {
    #[cfg(not(feature = "backup"))]
    {
        println!("{} Backup feature not enabled", "✗".red());
        println!("Rebuild with: cargo build --features backup");
        return Ok(());
    }

    #[cfg(feature = "backup")]
    {
        if verbose {
            println!("{}", "Running restore in verbose mode".dimmed());
        }

        println!(
            "\n{}",
            "┌─ Restore from encrypted backup ─────────────────────┐".cyan()
        );
        println!(
            "{}\n",
            "└──────────────────────────────────────────────────────┘".cyan()
        );

        // Read backup file
        let encrypted =
            fs::read_to_string(&backup).with_context(|| format!("Failed to read {}", backup))?;

        println!("{} Read backup from {}", "✓".green(), backup);

        // Check if output exists
        if std::path::Path::new(&output).exists() {
            if !Confirm::new()
                .with_prompt(format!("{} already exists. Overwrite?", output))
                .default(false)
                .interact()?
            {
                println!("{} Restore cancelled", "ℹ️".cyan());
                return Ok(());
            }
        }

        // Get password
        let password = Password::new()
            .with_prompt("Enter decryption password")
            .interact()?;

        println!("{} Attempting decryption...", "ℹ️".cyan());

        // Decrypt
        let plaintext = match decrypt_content(&encrypted, &password) {
            Ok(p) => p,
            Err(e) => {
                println!("{} Decryption failed: {}", "✗".red(), e);
                println!("\nPossible reasons:");
                println!("  • Incorrect password");
                println!("  • Corrupted backup file");
                println!("  • Incompatible backup version");
                return Err(e);
            }
        };

        println!("{} Decryption successful", "✓".green());

        // Write restored file
        fs::write(&output, &plaintext).with_context(|| format!("Failed to write to {}", output))?;

        println!("{} Restored to {}", "✓".green(), output);
        println!("\n{}", "⚠️  Remember to:".yellow().bold());
        println!("  • Set secure permissions: chmod 600 {}", output);
        println!("  • Verify the contents before using");
        println!("  • Delete backup if no longer needed");

        Ok(())
    }
}

#[cfg(feature = "backup")]
fn decrypt_content(encrypted_b64: &str, password: &str) -> Result<String> {
    // Decode base64
    let encrypted = general_purpose::STANDARD
        .decode(encrypted_b64)
        .context("Invalid base64 encoding")?;

    // Parse format: version(1) || salt(32) || nonce(12) || ciphertext
    if encrypted.len() < 45 {
        return Err(anyhow!("Invalid backup file format"));
    }

    let version = encrypted[0];
    if version != 1 {
        return Err(anyhow!("Unsupported backup version: {}", version));
    }

    let salt = &encrypted[1..33];
    let nonce_bytes = &encrypted[33..45];
    let ciphertext = &encrypted[45..];

    // Derive key from password
    let argon2 = Argon2::default();
    let binding = salt_string(salt);
    let password_hash = argon2
        .hash_password(password.as_bytes(), &binding)
        .map_err(|e| anyhow!("Failed to derive key: {}", e))?;

    let key_bytes = password_hash.hash.unwrap();
    let key = key_bytes.as_bytes();

    if key.len() < 32 {
        return Err(anyhow!("Derived key too short"));
    }
    let key: &[u8; 32] = key[..32].try_into()?;

    // Create cipher
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("Decryption failed - incorrect password or corrupted file"))?;

    // Convert to string
    String::from_utf8(plaintext).context("Invalid UTF-8 in decrypted content")
}

#[cfg(feature = "backup")]
fn salt_string(salt: &[u8]) -> argon2::password_hash::SaltString {
    use argon2::password_hash::SaltString;
    SaltString::encode_b64(salt).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "backup")]
    fn test_decrypt_invalid_format() {
        let result = decrypt_content("invalid", "password");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(feature = "backup")]
    fn test_decrypt_short_data() {
        let result = decrypt_content("YWJjZA==", "password"); // "abcd" in base64
        assert!(result.is_err());
    }
}
