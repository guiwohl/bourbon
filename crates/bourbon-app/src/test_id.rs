use gpui::InteractiveElement;

pub(crate) trait TestSupportExt: InteractiveElement + Sized {
    fn test_support(mut self) -> Self {
        let id = self.interactivity().element_id.clone();
        self.debug_selector(move || id.map(|id| id.to_string()).unwrap_or_default())
    }
}

impl<E: InteractiveElement> TestSupportExt for E {}
