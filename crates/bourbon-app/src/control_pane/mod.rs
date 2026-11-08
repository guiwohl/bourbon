mod actions;
mod highlight;
mod query;
mod render;

use bourbon_core::{NoteStore, SearchHit, StoreError};
use bourbon_editor::{TextField, TextFieldEvent};
use bourbon_ui::ThemeExt as _;
use gpui::{AppContext as _, Context, Entity, Subscription, Window, actions};

use crate::workspace::Workspace;

pub(crate) use actions::PaneAction;
pub(crate) use query::{ACTIONS_QUERY, Mode};
pub(crate) use render::render;

actions!(bourbon_control_pane, [HighlightNext, HighlightPrevious]);

pub(crate) const KEY_CONTEXT: &str = "ControlPane";

const SEARCH_LIMIT: usize = 50;

pub(crate) enum Results {
    Notes(Vec<SearchHit>),
    Actions(Vec<PaneAction>),
}

pub(crate) struct ControlPane {
    input: Entity<TextField>,
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
        let input_size = cx.theme().tokens.scale.big;
        let input = cx.new(|cx| {
            TextField::new(cx)
                .placeholder("Search notes or type > for actions")
                .borderless()
                .text_size(input_size)
        });
        input.update(cx, |input, cx| {
            input.set_text(initial_query, cx);
            input.focus(window, cx);
        });
        let subscription = cx.subscribe_in(
            &input,
            window,
            |workspace, _, event: &TextFieldEvent, window, cx| match event {
                TextFieldEvent::Changed(_) => workspace.control_pane_query_changed(cx),
                TextFieldEvent::Submit => workspace.activate_highlighted_row(window, cx),
                TextFieldEvent::Cancel => workspace.close_control_pane(window, cx),
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
        self.input.read(cx).text()
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

    pub(crate) fn highlight(&mut self, row: usize) {
        self.highlighted = row;
    }

    pub(crate) fn input(&self) -> &Entity<TextField> {
        &self.input
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
