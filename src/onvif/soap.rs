//! SOAP envelope parsing and response construction.
//!
//! Requests are parsed with a real XML parser so that namespace prefixes,
//! attributes, comments and whitespace never influence which operation is
//! detected or how credentials are read.

use roxmltree::{Document, Node};

/// SOAP 1.2 envelope namespace.
pub const SOAP12_NS: &str = "http://www.w3.org/2003/05/soap-envelope";
/// SOAP 1.1 envelope namespace (accepted on input for lenient clients).
pub const SOAP11_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";
/// ONVIF error namespace.
pub const ONVIF_ERROR_NS: &str = "http://www.onvif.org/ver10/error";

/// How the password inside a WS-Security UsernameToken is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordType {
    Text,
    Digest,
}

/// A WS-Security UsernameToken extracted from the SOAP header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsernameToken {
    pub username: String,
    pub password: String,
    pub password_type: PasswordType,
    pub nonce: Option<String>,
    pub created: Option<String>,
}

/// A parsed SOAP request.
pub struct SoapRequest<'a> {
    doc: Document<'a>,
}

impl<'a> SoapRequest<'a> {
    /// Parses a SOAP envelope. Fails on malformed XML or a missing Body.
    pub fn parse(xml: &'a str) -> Result<Self, String> {
        let doc = Document::parse(xml).map_err(|e| format!("malformed XML: {e}"))?;
        let req = SoapRequest { doc };
        if req.body_node().is_none() {
            return Err("SOAP Body element not found".to_string());
        }
        Ok(req)
    }

    fn is_soap_ns(ns: Option<&str>) -> bool {
        matches!(ns, Some(SOAP12_NS) | Some(SOAP11_NS) | None)
    }

    fn body_node(&self) -> Option<Node<'_, 'a>> {
        self.doc.root_element().children().find(|n| {
            n.is_element()
                && n.tag_name().name() == "Body"
                && Self::is_soap_ns(n.tag_name().namespace())
        })
    }

    fn header_node(&self) -> Option<Node<'_, 'a>> {
        self.doc
            .root_element()
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "Header")
    }

    /// The first element inside the Body, i.e. the operation being invoked.
    pub fn action_node(&self) -> Option<Node<'_, 'a>> {
        self.body_node()?.children().find(|n| n.is_element())
    }

    /// Local name of the requested operation, e.g. `GetProfiles`.
    pub fn action(&self) -> Option<&str> {
        self.action_node().map(|n| n.tag_name().name())
    }

    /// Text of a direct child element of the operation element, by local name.
    pub fn action_parameter(&self, local_name: &str) -> Option<String> {
        self.action_node()?
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == local_name)
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
    }

    /// Extracts the WS-Security UsernameToken from the header, if present.
    pub fn username_token(&self) -> Option<UsernameToken> {
        let header = self.header_node()?;
        let security = header
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "Security")?;
        let token = security
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "UsernameToken")?;

        let child_text = |name: &str| -> Option<String> {
            token
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == name)
                .and_then(|n| n.text())
                .map(|t| t.trim().to_string())
        };

        let username = child_text("Username")?;
        let password_node = token
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "Password")?;
        let password = password_node.text().unwrap_or("").trim().to_string();
        let password_type = match password_node.attribute("Type") {
            Some(t) if t.ends_with("#PasswordDigest") => PasswordType::Digest,
            _ => PasswordType::Text,
        };

        Some(UsernameToken {
            username,
            password,
            password_type,
            nonce: child_text("Nonce"),
            created: child_text("Created"),
        })
    }
}

/// Escapes text for safe inclusion as XML character data or attribute value.
pub fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Builds a SOAP 1.2 envelope around a header and body fragment.
#[derive(Debug, Clone)]
pub struct SoapResponseBuilder {
    header_content: String,
    body_content: String,
    namespaces: Vec<(String, String)>,
}

impl Default for SoapResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SoapResponseBuilder {
    pub fn new() -> Self {
        Self {
            header_content: String::new(),
            body_content: String::new(),
            namespaces: vec![("soap".to_string(), SOAP12_NS.to_string())],
        }
    }

    pub fn add_namespace(mut self, prefix: &str, uri: &str) -> Self {
        self.namespaces.push((prefix.to_string(), uri.to_string()));
        self
    }

    pub fn set_header(mut self, content: &str) -> Self {
        self.header_content = content.to_string();
        self
    }

    pub fn set_body(mut self, content: &str) -> Self {
        self.body_content = content.to_string();
        self
    }

    pub fn build(self) -> String {
        let mut namespaces = String::new();
        for (prefix, uri) in &self.namespaces {
            namespaces.push_str(&format!(" xmlns:{prefix}=\"{uri}\""));
        }

        let header = if self.header_content.is_empty() {
            String::new()
        } else {
            format!("<soap:Header>{}</soap:Header>\n", self.header_content)
        };

        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<soap:Envelope{namespaces}>\n{header}<soap:Body>\n{}\n</soap:Body>\n</soap:Envelope>",
            self.body_content
        )
    }
}

/// SOAP 1.2 fault code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCode {
    Sender,
    Receiver,
}

/// Builds a SOAP 1.2 fault with an ONVIF subcode (e.g. `ter:NotAuthorized`).
pub fn soap_fault(code: FaultCode, subcode: &str, reason: &str) -> String {
    let code_text = match code {
        FaultCode::Sender => "soap:Sender",
        FaultCode::Receiver => "soap:Receiver",
    };
    let body = format!(
        "<soap:Fault>\n<soap:Code>\n<soap:Value>{code_text}</soap:Value>\n<soap:Subcode>\n<soap:Value>ter:{subcode}</soap:Value>\n</soap:Subcode>\n</soap:Code>\n<soap:Reason>\n<soap:Text xml:lang=\"en\">{}</soap:Text>\n</soap:Reason>\n</soap:Fault>",
        xml_escape(reason)
    );
    SoapResponseBuilder::new()
        .add_namespace("ter", ONVIF_ERROR_NS)
        .set_body(&body)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WSSE_REQUEST: &str = r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
  xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"
  xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">
  <s:Header>
    <wsse:Security s:mustUnderstand="1">
      <wsse:UsernameToken>
        <wsse:Username>admin</wsse:Username>
        <wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">abc=</wsse:Password>
        <wsse:Nonce EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">bm9uY2U=</wsse:Nonce>
        <wsu:Created>2024-01-01T00:00:00Z</wsu:Created>
      </wsse:UsernameToken>
    </wsse:Security>
  </s:Header>
  <s:Body>
    <trt:GetStreamUri xmlns:trt="http://www.onvif.org/ver10/media/wsdl">
      <trt:ProfileToken>Profile_1</trt:ProfileToken>
    </trt:GetStreamUri>
  </s:Body>
</s:Envelope>"#;

    #[test]
    fn extracts_action_and_parameters() {
        let req = SoapRequest::parse(WSSE_REQUEST).unwrap();
        assert_eq!(req.action(), Some("GetStreamUri"));
        assert_eq!(
            req.action_parameter("ProfileToken").as_deref(),
            Some("Profile_1")
        );
        assert_eq!(req.action_parameter("Missing"), None);
    }

    #[test]
    fn extracts_prefixed_username_token() {
        let req = SoapRequest::parse(WSSE_REQUEST).unwrap();
        let token = req.username_token().expect("token");
        assert_eq!(token.username, "admin");
        assert_eq!(token.password, "abc=");
        assert_eq!(token.password_type, PasswordType::Digest);
        assert_eq!(token.nonce.as_deref(), Some("bm9uY2U="));
        assert_eq!(token.created.as_deref(), Some("2024-01-01T00:00:00Z"));
    }

    #[test]
    fn extracts_unprefixed_text_password() {
        let xml = r#"<Envelope xmlns="http://www.w3.org/2003/05/soap-envelope"><Header><Security><UsernameToken><Username>u</Username><Password>p</Password></UsernameToken></Security></Header><Body><GetProfiles/></Body></Envelope>"#;
        let req = SoapRequest::parse(xml).unwrap();
        let token = req.username_token().unwrap();
        assert_eq!(token.username, "u");
        assert_eq!(token.password, "p");
        assert_eq!(token.password_type, PasswordType::Text);
        assert_eq!(req.action(), Some("GetProfiles"));
    }

    #[test]
    fn operation_names_in_comments_or_headers_are_ignored() {
        let xml = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Header><!-- GetCapabilities --><Foo>GetDeviceInformation</Foo></s:Header><s:Body><GetStreamUri/></s:Body></s:Envelope>"#;
        let req = SoapRequest::parse(xml).unwrap();
        assert_eq!(req.action(), Some("GetStreamUri"));
    }

    #[test]
    fn rejects_malformed_xml_and_missing_body() {
        assert!(SoapRequest::parse("<s:Envelope>").is_err());
        assert!(SoapRequest::parse("<root><x/></root>").is_err());
    }

    #[test]
    fn builder_and_fault_produce_well_formed_xml() {
        let out = SoapResponseBuilder::new()
            .add_namespace("tds", "http://www.onvif.org/ver10/device/wsdl")
            .set_body("<tds:GetHostnameResponse/>")
            .build();
        let doc = Document::parse(&out).unwrap();
        assert_eq!(doc.root_element().tag_name().name(), "Envelope");

        let fault = soap_fault(FaultCode::Sender, "NotAuthorized", "a < b & \"c\"");
        let doc = Document::parse(&fault).unwrap();
        assert!(doc.descendants().any(|n| n.tag_name().name() == "Fault"));
        assert!(fault.contains("ter:NotAuthorized"));
    }

    #[test]
    fn xml_escape_handles_special_characters() {
        assert_eq!(xml_escape("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&apos;");
    }
}
