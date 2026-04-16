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

    /// query가 비어있으면 전체 인덱스를 순서대로, 매칭 위치는 빈 Vec으로 반환.
    /// 아닌 경우 매칭되는 인덱스를 점수 내림차순으로, 각 아이템의 매칭된 글자 위치(char index)도 함께 반환.
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

    fn indices(results: &[(usize, Vec<u32>)]) -> Vec<usize> {
        results.iter().map(|(i, _)| *i).collect()
    }

    #[test]
    fn test_empty_query_returns_all_in_order() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let result = f.filter_with_indices("", &items);
        assert_eq!(indices(&result), vec![0, 1, 2]);
        assert!(result.iter().all(|(_, m)| m.is_empty()));
    }

    #[test]
    fn test_fuzzy_match_includes_matching_item() {
        let mut f = FuzzyFilter::new();
        let items = vec!["editor".to_string(), "terminal".to_string(), "server".to_string()];
        let result = f.filter_with_indices("ed", &items);
        assert!(indices(&result).contains(&0), "editor should match 'ed'");
    }

    #[test]
    fn test_no_match_returns_empty() {
        let mut f = FuzzyFilter::new();
        let items = vec!["alpha".to_string(), "beta".to_string()];
        let result = f.filter_with_indices("zzz", &items);
        assert!(result.is_empty());
    }

    #[test]
    fn test_filter_with_empty_items() {
        let mut f = FuzzyFilter::new();
        let result = f.filter_with_indices("abc", &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_higher_score_appears_first() {
        let mut f = FuzzyFilter::new();
        let items = vec!["embedded".to_string(), "editor".to_string()];
        let result = f.filter_with_indices("edi", &items);
        assert_eq!(indices(&result)[0], 1, "editor should rank higher for 'edi'");
    }

    #[test]
    fn test_match_indices_non_empty_for_query() {
        let mut f = FuzzyFilter::new();
        let items = vec!["editor".to_string()];
        let result = f.filter_with_indices("ed", &items);
        assert!(!result[0].1.is_empty(), "match positions should be non-empty");
    }
}
