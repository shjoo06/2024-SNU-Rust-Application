// builder pattern: used to construct a complex object step by step
// PackageBuilder: 거쳐가는 struct: 최종적으로는 Package에 모든 값의 소유권을 넘기고, 본인은 빈껍데기가 되어 사라진다.

#[derive(Debug)]
enum Language {
    Rust,
    Java,
    Perl,
}

#[derive(Clone, Debug)]
struct Dependency {
    name: String,
    version_expression: String,
}

// A representation of a software package.
#[derive(Debug)]
struct Package {
    name: String,
    version: String,
    authors: Vec<String>,
    dependencies: Vec<Dependency>,
    language: Option<Language>,
}

impl Package {
    // Return a representation of this package as a dependency, for use in
    // building other packages.
    fn as_dependency(&self) -> Dependency { // Package should not be consumed here
        Dependency {
            name: self.name.clone(),
            version_expression: self.version.clone(),
        }
    }
}

// A builder for a Package. Use `build()` to create the `Package` itself.
struct PackageBuilder(Package); // tuple struct with one field

impl PackageBuilder {
    fn new(name: impl Into<String>) -> Self {
        PackageBuilder(Package {
            name: name.into(),
            version: String::new(),
            authors: Vec::new(),
            dependencies: Vec::new(),
            language: None,
        })
    
    }

    // Set the package version.
    fn version(mut self, version: impl Into<String>) -> Self {
        self.0.version = version.into();
        self
    }

    // Set the package authors.
    fn authors(mut self, authors: Vec<String>) -> Self {
        self.0.authors.extend(authors.into_iter().map(Into::into)); // extend<I>(&mut self, iter: I), I: IntoIterator<Item=T>
        self
    }

    // Add an additional dependency.
    fn dependency(mut self, dependency: Dependency) -> Self {
        self.0.dependencies.push(dependency);
        self
    }

    // Set the language. If not set, language defaults to None.
    fn language(mut self, language: Language) -> Self {
        self.0.language = Some(language);
        self
    }

    fn build(self) -> Package { // consumes the builder(self), returns the constructed Package
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_base64() {
        let base64 = PackageBuilder::new("base64").version("0.13").build();
        assert_eq!(format!("base64: {:?}", base64), "base64: Package { name: \"base64\", version: \"0.13\", authors: [], dependencies: [], language: None }");
    }

    #[test]
    fn builder_log() {
        let log = PackageBuilder::new("log")
            .version("0.4")
            .language(Language::Rust)
            .build();
        assert_eq!(format!("log: {:?}", log), "log: Package { name: \"log\", version: \"0.4\", authors: [], dependencies: [], language: Some(Rust) }");
    }

    #[test]
    fn builder_serde() {
        let base64 = PackageBuilder::new("base64").version("0.13").build();
        let log = PackageBuilder::new("log")
            .version("0.4")
            .language(Language::Rust)
            .build();
        let serde = PackageBuilder::new("serde")
            .authors(vec!["djmitche".into()])
            .version(String::from("4.0"))
            .dependency(base64.as_dependency())
            .dependency(log.as_dependency())
            .build();
        assert_eq!(format!("serde: {:?}", serde), "serde: Package { name: \"serde\", version: \"4.0\", authors: [\"djmitche\"], dependencies: [Dependency { name: \"base64\", version_expression: \"0.13\" }, Dependency { name: \"log\", version_expression: \"0.4\" }], language: None }");
    }
}
