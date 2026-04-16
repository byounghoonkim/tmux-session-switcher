use nucleo_matcher::{Config, Matcher, Utf32Str};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};

pub(crate) struct FuzzyFilter {
    matcher: Matcher,
}

impl FuzzyFilter {
    pub(crate) fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    /// query가 비어있으면 전체 인덱스를 순서대로 반환.
    /// 아닌 경우 매칭되는 인덱스를 점수 내림차순으로 반환.
    pub(crate) fn filter(&mut self, query: &str, items: &[String]) -> Vec<usize> {
        if query.is_empty() {
            return (0..items.len()).collect();
        }
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u32)> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let score = pattern.score(Utf32Str::new(item, &mut buf), &mut self.matcher)?;
                Some((i, score))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(i, _)| i).collect()
    }

    /// filter와 동일하지만 각 아이템의 매칭된 글자 위치(char index)도 함께 반환.
    /// 쿼리가 비어있으면 매칭 위치는 빈 Vec.
    pub(crate) fn filter_with_indices(
        &mut self,
        query: &str,
        items: &[String],
    ) -> Vec<(usize, Vec<u32>)> {
        if query.is_empty() {
            return (0..items.len()).map(|i| (i, vec![])).collect();
        }
        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut buf = Vec::new();
        let mut idx_buf = Vec::new();
        let mut scored: Vec<(usize, u32, Vec<u32>)> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                idx_buf.clear();
                let score = pattern.indices(
                    Utf32Str::new(item, &mut buf),
                    &mut self.matcher,
                    &mut idx_buf,
                )?;
                idx_buf.sort_unstable();
                Some((i, score, idx_buf.clone()))
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(i, _, positions)| (i, positions)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query_returns_all_in_order() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let result = f.filter("", &items);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_fuzzy_match_includes_matching_item() {
        let mut f = FuzzyFilter::new();
        let items = vec!["editor".to_string(), "terminal".to_string(), "server".to_string()];
        let result = f.filter("ed", &items);
        assert!(result.contains(&0), "editor should match 'ed'");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string()];
        let result = f.filter("zzz", &items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_with_empty_items() {
        let mut f = FuzzyFilter::new();
        let result = f.filter("abc", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_higher_score_appears_first() {
        let mut f = FuzzyFilter::new();
        // "ed" should match "editor" with higher score than "embedded"
        let items = vec!["embedded".to_string(), "editor".to_string()];
        let result = f.filter("edi", &items);
        // "editor" (index 1) should score higher than "embedded" (index 0) for "edi"
        assert_eq!(result[0], 1, "editor should rank higher for 'edi'");
    }
}
