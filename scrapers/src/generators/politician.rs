use slugify::slugify;

pub struct PoliticianSlugGenerator<'a> {
    pub name: &'a str,
}

impl<'a> PoliticianSlugGenerator<'a> {
    pub fn new(name: &'a str) -> Self {
        PoliticianSlugGenerator { name }
    }

    pub fn generate(&self) -> String {
        slugify!(self.name)
    }
}

pub struct PoliticianRefKeyGenerator<'a> {
    pub source: &'a str,
    pub slug: &'a str,
}

impl<'a> PoliticianRefKeyGenerator<'a> {
    pub fn new(source: &'a str, slug: &'a str) -> Self {
        PoliticianRefKeyGenerator { source, slug }
    }

    pub fn generate(&self) -> String {
        slugify!(&format!("{} {}", self.source, self.slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn politician_slug() {
        let tests: Vec<(&'static str, &'static str)> = vec![("John Smith", "john-smith")];

        for (input, expected) in tests {
            assert_eq!(PoliticianSlugGenerator::new(input).generate(), expected);
        }
    }

    #[test]
    fn politician_ref_key() {
        let tests: Vec<((&'static str, &'static str), &'static str)> = vec![
            (("CO SOS", "john-smith"), "co-sos-john-smith"),
            (("MN CSV", "john-smith"), "mn-csv-john-smith"),
        ];

        for (input, expected) in tests {
            assert_eq!(
                PoliticianRefKeyGenerator::new(input.0, input.1).generate(),
                expected
            );
        }
    }
}
