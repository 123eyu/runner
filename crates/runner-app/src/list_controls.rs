use runner_core::protocol::ListPage;

use runner_app::ui::list::normalize_page_size;
use runner_app::ui::{clamp_page, DEFAULT_PAGE_SIZE};

pub(crate) const LIST_QUERY_DEBOUNCE_MS: u64 = 200;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ListRequest {
    pub request_id: u64,
    pub page: usize,
    pub page_size: usize,
    pub query: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct QueryUpdate {
    pub generation: u64,
    pub load_now: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ListControls<T> {
    pub query: String,
    pub debounced_query: String,
    pub page: usize,
    pub items: Vec<T>,
    pub page_size: usize,
    pub filtered_count: usize,
    pub total_count: usize,
    pub loaded: bool,
    pub loading: bool,
    pub error: Option<String>,
    request_id: u64,
    debounce_generation: u64,
}

impl<T> Default for ListControls<T> {
    fn default() -> Self {
        Self {
            query: String::new(),
            debounced_query: String::new(),
            page: 1,
            page_size: DEFAULT_PAGE_SIZE,
            items: Vec::new(),
            filtered_count: 0,
            total_count: 0,
            loaded: false,
            loading: true,
            error: None,
            request_id: 0,
            debounce_generation: 0,
        }
    }
}

impl<T> ListControls<T> {
    pub fn reset(&mut self) {
        self.query.clear();
        self.debounced_query.clear();
        self.page = 1;
        self.invalidate();
        self.request_id = self.request_id.wrapping_add(1);
        self.debounce_generation = self.debounce_generation.wrapping_add(1);
    }

    pub fn set_load_error(&mut self, error: Option<String>) {
        self.loading = error.is_none();
        self.error = error;
    }

    pub fn invalidate(&mut self) {
        self.items.clear();
        self.filtered_count = 0;
        self.total_count = 0;
        self.loaded = false;
        self.set_load_error(None);
    }

    /// Whether the loaded counts answer a search rather than the whole list.
    pub fn searching(&self) -> bool {
        !self.debounced_query.trim().is_empty()
    }

    pub fn page_count(&self) -> usize {
        self.filtered_count.div_ceil(self.page_size)
    }

    pub fn set_query(&mut self, query: String) -> QueryUpdate {
        self.query = query;
        self.page = 1;
        self.debounce_generation = self.debounce_generation.wrapping_add(1);
        QueryUpdate {
            generation: self.debounce_generation,
            load_now: self.query == self.debounced_query,
        }
    }

    pub fn apply_debounced_query(&mut self, generation: u64) -> bool {
        if generation != self.debounce_generation || self.debounced_query == self.query {
            return false;
        }
        self.debounced_query.clone_from(&self.query);
        true
    }

    pub fn set_page(&mut self, page: usize) -> bool {
        let next = clamp_page(page, self.page_count());
        if next == self.page {
            return false;
        }
        self.page = next;
        true
    }

    pub fn set_page_size(&mut self, size: usize) -> bool {
        let size = normalize_page_size(size);
        if self.page_size == size {
            return false;
        }
        self.page_size = size;
        self.page = 1;
        self.request_id = self.request_id.wrapping_add(1);
        true
    }

    pub fn begin_load(&mut self) -> ListRequest {
        self.request_id = self.request_id.wrapping_add(1);
        self.loading = true;
        self.error = None;
        ListRequest {
            request_id: self.request_id,
            page: self.page,
            page_size: self.page_size,
            query: self.debounced_query.clone(),
        }
    }

    pub fn apply_success(&mut self, request_id: u64, result: ListPage<T>) -> bool {
        if request_id != self.request_id {
            return false;
        }
        self.items = result.items;
        self.filtered_count = count_to_usize(result.filtered_count);
        self.total_count = count_to_usize(result.total_count);
        self.loaded = true;
        self.loading = false;
        let next_page = clamp_page(self.page, self.page_count());
        let page_changed = next_page != self.page;
        self.page = next_page;
        page_changed
    }

    pub fn apply_error(&mut self, request_id: u64, error: String) {
        if request_id != self.request_id {
            return;
        }
        self.error = Some(error);
        self.loading = false;
    }
}

fn count_to_usize(count: i64) -> usize {
    usize::try_from(count.max(0)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(items: Vec<&str>, filtered_count: i64, total_count: i64) -> ListPage<String> {
        ListPage {
            items: items.into_iter().map(str::to_owned).collect(),
            filtered_count,
            total_count,
        }
    }

    #[test]
    fn search_resets_page_and_only_latest_debounce_commits() {
        let mut controls = ListControls::<String> {
            page: 3,
            filtered_count: 30,
            ..Default::default()
        };
        let stale = controls.set_query("need".into());
        let latest = controls.set_query("needle".into());

        assert_eq!(controls.page, 1);
        assert!(!stale.load_now);
        assert!(!controls.apply_debounced_query(stale.generation));
        assert_eq!(controls.debounced_query, "");
        assert!(controls.apply_debounced_query(latest.generation));
        assert_eq!(controls.debounced_query, "needle");
    }

    #[test]
    fn searching_follows_the_committed_query() {
        let mut controls = ListControls::<String>::default();
        assert!(!controls.searching());
        let update = controls.set_query("  ".into());
        assert!(controls.apply_debounced_query(update.generation));
        assert!(!controls.searching(), "a blank query lists everything");
        let update = controls.set_query("rev".into());
        assert!(
            !controls.searching(),
            "counts still answer the committed query"
        );
        assert!(controls.apply_debounced_query(update.generation));
        assert!(controls.searching());
    }

    #[test]
    fn reverting_to_the_committed_query_loads_the_reset_page_immediately() {
        let mut controls = ListControls::<String> {
            page: 3,
            filtered_count: 30,
            ..Default::default()
        };
        let pending = controls.set_query("needle".into());
        let reverted = controls.set_query(String::new());

        assert!(!pending.load_now);
        assert!(reverted.load_now);
        assert_eq!(controls.page, 1);
        assert!(!controls.apply_debounced_query(pending.generation));
    }

    #[test]
    fn success_updates_counts_and_requests_reload_when_page_disappears() {
        let mut controls = ListControls::<String> {
            page: 2,
            filtered_count: 11,
            total_count: 11,
            ..Default::default()
        };
        let request = controls.begin_load();
        let reload = controls.apply_success(request.request_id, page(vec!["one"], 10, 10));

        assert!(reload);
        assert_eq!(controls.page, 1);
        assert_eq!(controls.items, ["one"]);
        assert_eq!(controls.filtered_count, 10);
        assert_eq!(controls.total_count, 10);
        assert_eq!(controls.page_count(), 1);
        assert!(controls.loaded);
        assert!(!controls.loading);
    }

    #[test]
    fn stale_results_do_not_replace_the_current_page() {
        let mut controls = ListControls::<String>::default();
        let stale = controls.begin_load();
        let current = controls.begin_load();

        assert!(!controls.apply_success(stale.request_id, page(vec!["stale"], 1, 1)));
        assert!(controls.items.is_empty());
        assert!(!controls.apply_success(current.request_id, page(vec!["current"], 1, 1)));
        assert_eq!(controls.items, ["current"]);
    }

    #[test]
    fn page_changes_are_clamped_to_the_filtered_count() {
        let mut controls = ListControls::<String> {
            page: 1,
            filtered_count: 21,
            ..Default::default()
        };
        assert!(controls.set_page(99));
        assert_eq!(controls.page, 3);
        assert!(!controls.set_page(3));
    }

    #[test]
    fn invalidate_reloads_library_without_resetting_navigation() {
        let mut controls = ListControls {
            query: "needle".into(),
            debounced_query: "needle".into(),
            page: 2,
            page_size: 25,
            items: vec!["old"],
            filtered_count: 30,
            total_count: 50,
            loaded: true,
            loading: false,
            error: Some("old error".into()),
            ..Default::default()
        };
        controls.invalidate();
        assert!(controls.items.is_empty());
        assert_eq!((controls.filtered_count, controls.total_count), (0, 0));
        assert!(!controls.loaded);
        assert!(controls.loading);
        assert_eq!(controls.error, None);
        assert_eq!((controls.page, controls.page_size), (2, 25));
        assert_eq!(controls.query, "needle");
        assert_eq!(controls.debounced_query, "needle");
        controls.set_load_error(Some("failed".into()));
        assert!(!controls.loading);
        assert_eq!(controls.error.as_deref(), Some("failed"));
    }

    #[test]
    fn reset_restores_mount_state_and_invalidates_pending_work() {
        let mut controls = ListControls::<String> {
            page: 2,
            filtered_count: 9,
            total_count: 12,
            loaded: true,
            loading: false,
            error: Some("old error".into()),
            ..Default::default()
        };
        let debounce = controls.set_query("needle".into());
        let request = controls.begin_load();

        controls.reset();

        assert_eq!(controls.query, "");
        assert_eq!(controls.debounced_query, "");
        assert_eq!(controls.page, 1);
        assert!(controls.items.is_empty());
        assert_eq!(controls.filtered_count, 0);
        assert_eq!(controls.total_count, 0);
        assert!(!controls.loaded);
        assert!(controls.loading);
        assert_eq!(controls.error, None);
        assert!(!controls.apply_debounced_query(debounce.generation));
        assert!(!controls.apply_success(request.request_id, page(vec!["stale"], 1, 1)));
    }

    #[test]
    fn page_size_change_preserves_search_and_invalidates_old_results() {
        let mut controls = ListControls::<String>::default();
        assert_eq!(controls.begin_load().page_size, 10);
        let query = controls.set_query("review".into());
        controls.apply_debounced_query(query.generation);
        controls.filtered_count = 61;
        controls.total_count = 100;
        assert!(controls.set_page(3));
        let stale = controls.begin_load();

        assert!(controls.set_page_size(25));
        assert_eq!(controls.page, 1);
        assert_eq!(controls.page_count(), 3);
        assert_eq!(controls.query, "review");
        assert!(!controls.apply_success(stale.request_id, page(vec!["stale"], 61, 100)));
        controls.apply_error(stale.request_id, "stale error".into());
        assert!(controls.error.is_none());
        let request = controls.begin_load();
        assert_eq!(
            (request.page, request.page_size, request.query.as_str()),
            (1, 25, "review")
        );
        assert!(!controls.set_page_size(25));
        assert!(controls.set_page_size(50));
        assert_eq!(controls.page_count(), 2);
        controls.reset();
        assert_eq!(controls.begin_load().page_size, 50);
    }
}
