use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email", s))
        }
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // We just forward to the Display implementation of
        // the wrapped String.
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    // We are importing the `SafeEmail` faker!
    // We also need the `Fake` trait to get access to the
    // `.fake` method on `SafeEmail`

    // This is `property-based testing` where we test random
    // generated emails instead of a single hard-coded email
    // This does increase our confidence in the correctness
    // of the code
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail().fake();
        claims::assert_ok!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    // Fails with type error ="the trait `rand_core::RngCore` is not implemented for `Gen`
    // Leave singular email generation test instead above

    // Both `Clone` and `Debug` are required by `quickcheck`
    // #[derive(Debug, Clone)]
    // struct ValidEmailFixture(pub String);

    // impl quickcheck::Arbitrary for ValidEmailFixture {
    //     fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    //         let email = SafeEmail().fake_with_rng(g);
    //         Self(email)
    //     }
    // }

    // #[quickcheck_macros::quickcheck]
    // fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
    //     SubscriberEmail::parse(valid_email.0).is_ok()
    // }
}
