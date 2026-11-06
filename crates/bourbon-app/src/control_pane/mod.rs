mod actions;
mod highlight;
mod query;
mod render;

use bourbon_core::{NoteStore, SearchHit, StoreError};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{AppContext as _, Context, Entity, Subscription, Window};

use crate::workspace::Workspace;

pub(crate) use actions::PaneAction;
pub(crate) use query::{ACTIONS_QUERY, Mode};
pub(crate) use render::render;

const SEARCH_LIMIT: usize = 50;

pub(crate) enum Results {
    Notes(Vec<SearchHit>),
    Actions(Vec<PaneAction>),
}

pub(crate) struct ControlPane {
    input: Entity<InputState>,
    highlighted: usize,
    results: Results,
    _subscription: Subscription,
}

impl ControlPane {
    pub(crate) fn open(
        initial_query: &str,
        store: &NoteStore,
        has_selected_note: bool,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Result<Self, StoreError> {
        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search notes or type > for actions")
        });
        input.update(cx, |input, cx| {
            input.set_value(initial_query.to_owned(), window, cx);
            input.focus(window, cx);
        });
        let subscription = cx.subscribe_in(
            &input,
            window,
            |workspace, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => workspace.control_pane_query_changed(cx),
                InputEvent::PressEnter { .. } => workspace.activate_highlighted_row(window, cx),
                _ => {}
            },
        );
        Ok(Self {
            highlighted: 0,
            results: search(initial_query, store, has_selected_note)?,
            input,
            _subscription: subscription,
        })
    }

    pub(crate) fn query(&self, cx: &Context<Workspace>) -> String {
        self.input.read(cx).value().to_string()
    }

    pub(crate) fn show(&mut self, results: Results) {
        self.results = results;
        self.highlighted = 0;
    }

    pub(crate) fn highlight_next(&mut self) {
        self.highlighted = highlight::next(self.highlighted, self.row_count());
    }

    pub(crate) fn highlight_previous(&mut self) {
        self.highlighted = highlight::previous(self.highlighted);
    }

    pub(crate) fn highlighted_row(&self) -> usize {
        self.highlighted
    }

    pub(crate) fn row_count(&self) -> usize {
        match &self.results {
            Results::Notes(hits) => hits.len(),
            Results::Actions(actions) => actions.len(),
        }
    }

    pub(crate) fn results(&self) -> &Results {
        &self.results
    }

    #[cfg(test)]
    pub(crate) fn mode(&self) -> Mode {
        match self.results {
            Results::Notes(_) => Mode::Notes,
            Results::Actions(_) => Mode::Actions,
        }
    }

    #[cfg(test)]
    pub(crate) fn note_titles(&self) -> Vec<String> {
        match &self.results {
            Results::Notes(hits) => hits.iter().map(|hit| hit.summary.title.clone()).collect(),
            Results::Actions(_) => Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn actions(&self) -> Vec<PaneAction> {
        match &self.results {
            Results::Notes(_) => Vec::new(),
            Results::Actions(actions) => actions.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn highlighted(&self) -> usize {
        self.highlighted
    }
}

pub(crate) fn search(
    query: &str,
    store: &NoteStore,
    has_selected_note: bool,
) -> Result<Results, StoreError> {
    let parsed = query::parse(query);
    Ok(match parsed.mode {
        Mode::Notes => Results::Notes(store.search_notes(parsed.text, SEARCH_LIMIT)?),
        Mode::Actions => Results::Actions(actions::matching(parsed.text, has_selected_note)),
    })
}
