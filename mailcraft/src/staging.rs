use sha2::{Digest, Sha256};

/// Redirects an email address to a staging inbox.
///
/// When sending emails in staging/development environments, you don't want to
/// accidentally email real users. This function redirects any email to a staging
/// inbox while preserving a unique identifier for the original recipient.
///
/// The original email is hashed (SHA-256) and inserted as a `+tag` in the
/// staging address, so each original recipient maps to a unique address.
///
/// # Examples
///
/// ```
/// use mailcraft::redirect_email_for_staging;
///
/// let redirected = redirect_email_for_staging("alice@client.com", "staging@myapp.com");
/// // Returns something like: staging+a1b2c3d4e5f6...@myapp.com
///
/// assert!(redirected.starts_with("staging+"));
/// assert!(redirected.ends_with("@myapp.com"));
/// ```
pub fn redirect_email_for_staging(email: &str, staging_email: &str) -> String {
    const MAX_LOCAL_PART_LEN: usize = 64;
    const MIN_HASH_LEN: usize = 8;

    let Some((raw_prefix, domain)) = staging_email.split_once('@') else {
        return staging_email.to_string();
    };

    let full_hash = format!("{:x}", Sha256::digest(email.as_bytes()));

    let max_prefix_len = MAX_LOCAL_PART_LEN.saturating_sub(1 + MIN_HASH_LEN);
    let prefix = &raw_prefix[..raw_prefix.len().min(max_prefix_len)];
    let available = MAX_LOCAL_PART_LEN.saturating_sub(prefix.len() + 1);
    let hash_len = available.min(full_hash.len());

    format!("{prefix}+{}@{domain}", &full_hash[..hash_len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_produces_unique_addresses() {
        let a = redirect_email_for_staging("alice@example.com", "staging@myapp.com");
        let b = redirect_email_for_staging("bob@example.com", "staging@myapp.com");

        assert_ne!(a, b);
        assert!(a.starts_with("staging+"));
        assert!(a.ends_with("@myapp.com"));
    }

    #[test]
    fn redirect_same_input_same_output() {
        let a = redirect_email_for_staging("alice@example.com", "staging@myapp.com");
        let b = redirect_email_for_staging("alice@example.com", "staging@myapp.com");

        assert_eq!(a, b);
    }

    #[test]
    fn redirect_respects_local_part_limit() {
        let result = redirect_email_for_staging("alice@example.com", "staging@myapp.com");
        let local_part = result.split('@').next().unwrap();
        assert!(local_part.len() <= 64);
    }

    #[test]
    fn redirect_invalid_staging_email_returns_as_is() {
        let result = redirect_email_for_staging("alice@example.com", "not-an-email");
        assert_eq!(result, "not-an-email");
    }

    #[test]
    fn redirect_long_prefix_still_has_hash() {
        let long_prefix = "p".repeat(63);
        let staging = format!("{long_prefix}@example.com");
        let a = redirect_email_for_staging("alice@example.com", &staging);
        let b = redirect_email_for_staging("bob@example.com", &staging);

        assert_ne!(a, b);

        let local_part = a.split('@').next().unwrap();
        assert!(local_part.len() <= 64);

        let hash_part = local_part.split('+').nth(1).unwrap();
        assert!(hash_part.len() >= 8);
    }
}
