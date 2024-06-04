use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
};

// We have this custom configuration for padding because Node's base64url implementation will omit padding,
// but the spec https://datatracker.ietf.org/doc/html/rfc4648#section-5 doesn't specify that padding MUST be omitted.
// Hence, Other implementations might include it, so we keep this relaxed.
pub const INDIFFERENT_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_decode_padding_mode(DecodePaddingMode::Indifferent)
    .with_encode_padding(false);
pub const URL_SAFE: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, INDIFFERENT_PAD);
