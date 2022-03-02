pub mod thesaures;

use scraper::ElementRef;

#[macro_export]
macro_rules! static_selector {
    ($s:expr) => {{
        static SELECTOR: ::once_cell::sync::OnceCell<::scraper::Selector> =
            ::once_cell::sync::OnceCell::new();
        SELECTOR.get_or_init(|| ::scraper::Selector::parse($s).unwrap())
    }};
}

trait ToDisplay {
    fn to_display(&self) -> String;
}

impl<'a> ToDisplay for ElementRef<'a> {
    fn to_display(&self) -> String {
        self.text().collect::<String>().trim().into()
    }
}
