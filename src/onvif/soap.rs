pub struct SoapResponseBuilder {
    header_content: String,
    body_content: String,
    namespaces: Vec<(String, String)>,
}

impl SoapResponseBuilder {
    pub fn new() -> Self {
        Self {
            header_content: String::new(),
            body_content: String::new(),
            namespaces: vec![(
                "soap".to_string(),
                "http://www.w3.org/2003/05/soap-envelope".to_string(),
            )],
        }
    }

    pub fn add_namespace(&mut self, prefix: &str, uri: &str) -> &mut Self {
        self.namespaces.push((prefix.to_string(), uri.to_string()));
        self
    }

    pub fn set_header(&mut self, content: &str) -> &mut Self {
        self.header_content = content.to_string();
        self
    }

    pub fn set_body(&mut self, content: &str) -> &mut Self {
        self.body_content = content.to_string();
        self
    }

    pub fn build(&self) -> String {
        let mut namespaces_str = String::new();
        for (prefix, uri) in &self.namespaces {
            namespaces_str.push_str(&format!(" xmlns:{}=\"{}\"", prefix, uri));
        }

        let header_section = if self.header_content.is_empty() {
            String::new()
        } else {
            format!("<soap:Header>{}</soap:Header>", self.header_content)
        };

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope{}>
{}
<soap:Body>
{}
</soap:Body>
</soap:Envelope>"#,
            namespaces_str, header_section, self.body_content
        )
    }
}
