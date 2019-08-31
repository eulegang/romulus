use crate::ast::*;

pub(crate) trait ScopeProvider {
    fn provides(&self) -> Vec<String>;
}

impl ScopeProvider for Selector {
    fn provides(&self) -> Vec<String> {
        use Selector::*;

        match self {
            Match(m) => m.provides(),
            Range(r) => r.provides(),
            Pattern(p) => p.provides(),
            Negate(_) => vec![],
            Conjunction(lh, rh) => {
                let mut vars = lh.provides();
                vars.extend(rh.provides());
                vars.dedup();
                vars
            }

            Disjunction(lh, rh) => {
                let mut common = Vec::new();
                let r_set = rh.provides();
                for l in lh.provides() {
                    if r_set.contains(&l) {
                        common.push(l);
                    }
                }

                common
            }
        }
    }
}

impl ScopeProvider for Match {
    fn provides(&self) -> Vec<String> {
        let mut results = Vec::new();

        if let Match::Regex(regex) = self {
            for o in regex.capture_names() {
                if let Some(name) = o {
                    results.push(name.to_string());
                }
            }
        }

        results
    }
}

impl ScopeProvider for Range {
    fn provides(&self) -> Vec<String> {
        self.0.provides()
    }
}

impl ScopeProvider for PatternMatch {
    fn provides(&self) -> Vec<String> {
        let mut result = Vec::new();
        for pat in &self.patterns {
            result.extend(pat.provides());
        }
        result
    }
}

impl ScopeProvider for Pattern {
    fn provides(&self) -> Vec<String> {
        use Pattern::*;

        match self {
            String(_, _) => Vec::new(),
            Identifier(s) => vec![s.clone()],
            Regex(regex) => {
                let mut results = Vec::new();
                for o in regex.capture_names() {
                    if let Some(name) = o {
                        results.push(name.to_string());
                    }
                }

                results
            }
        }
    }
}
