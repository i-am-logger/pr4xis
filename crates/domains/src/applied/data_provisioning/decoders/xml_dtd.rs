//! XML 1.0 Document Type Definition (DTD) decoder.
//!
//! Identifies a byte stream as an XML 1.0 DTD by checking for an
//! `<!ELEMENT` or `<!ATTLIST` markup declaration (W3C XML 1.0 Fifth
//! Edition §3.2 + §3.3, Bray et al. 2008). Full DTD parsing (the
//! production \[29\] `markupdecl` family) is intentionally not
//! implemented here — DTD-grounded vocabularies (currently
//! Global WordNet's WN-LMF 1.3) use the registered bytes for
//! schema-identity + hash-pinning, not for runtime validation.
//!
//! ## Citation
//!
//! - **Bray, T., Paoli, J., Sperberg-McQueen, C. M., Maler, E. &
//!   Yergeau, F.** (2008) *Extensible Markup Language (XML) 1.0
//!   (Fifth Edition)*, W3C Recommendation 26 November 2008. §2.8
//!   document type declaration, §3.2 element type declarations,
//!   §3.3 attribute-list declarations, §4 physical structures.

/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::XmlDtd;

/// True iff `bytes` contains an XML 1.0 markup declaration —
/// either `<!ELEMENT` (§3.2 element-type declaration) or
/// `<!ATTLIST` (§3.3 attribute-list declaration). Skips leading
/// whitespace, XML declarations (`<?xml ... ?>`), and comments
/// (`<!-- ... -->`) before the check.
#[must_use]
pub fn is_dtd(bytes: &[u8]) -> bool {
    // Cheap byte-level scan — no allocation. DTDs always carry at
    // least one element-type or attlist declaration; comments and
    // XML declarations may precede them.
    let text = match core::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return false,
    };
    text.contains("<!ELEMENT") || text.contains("<!ATTLIST")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_element_decl() {
        let dtd = br#"<?xml version="1.0"?>
<!ELEMENT root (#PCDATA)>"#;
        assert!(is_dtd(dtd));
    }

    #[test]
    fn recognises_attlist_decl() {
        let dtd = br#"<!ATTLIST root id ID #REQUIRED>"#;
        assert!(is_dtd(dtd));
    }

    #[test]
    fn rejects_plain_xml() {
        assert!(!is_dtd(b"<root/>"));
        assert!(!is_dtd(b""));
    }

    #[test]
    fn rejects_non_utf8() {
        assert!(!is_dtd(&[0xff, 0xfe, 0xfd]));
    }
}
