use unicode_segmentation::UnicodeSegmentation;

// This is a tuple struct
// Is a proper new type, not an alias - it does not inherit any of the methods available
// on String and trying to assign a String to a variable of type SubscriberName will trigger
// a compiler error
pub struct SubscriberName(String);

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

impl SubscriberName {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constrains on subscriber names.
    /// It panics otherwise.

    // `parse` is the only way to build an instance of `SubscriberName` outside of the domain module.
    // We can therefore assert that any instance of `SubscriberName` will satisfy all our validation constrains.
    // We have made it impossible for an instance of `SubscriberName` to violate those constrains.
    pub fn parse(s: String) -> SubscriberName {
        // `.trim() returns a view over the input `s` without trailing
        // whitespace-like characters.
        // `.is_empty` checks if the view contains any character
        let is_empty_or_whitespace = s.trim().is_empty();

        // A ghapheme is defined by the unicode standard as a "user-perceived"
        // character. `ñ` is a single grapheme, but it is composed of the 2 characters
        // (`n` and `~`)

        // `graphemes` returns an iterator over the graphemes in the input `s`
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        let is_too_long = s.graphemes(true).count() > 256;

        // Iterate over all characters iun the input `s` to check if any of them matches
        // one of the characters in the forbidden array.
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contain_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        // Return `false` if any of our conditions have been violated
        if is_empty_or_whitespace || is_too_long || contain_forbidden_characters {
            panic!("{} is not valid subscriber name.", s)
        } else {
            Self(s)
        }
    }
}
