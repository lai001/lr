use rust_i18n::Backend;

rust_i18n::i18n!("../../Resource/locales");

pub struct SharedBackend;

impl Backend for SharedBackend {
    fn available_locales(&self) -> Vec<&str> {
        _RUST_I18N_BACKEND.available_locales()
    }

    fn translate(&self, locale: &str, key: &str) -> Option<&str> {
        _RUST_I18N_BACKEND
            .translate(locale, key)
            .or_else(|| _RUST_I18N_BACKEND.translate("en", key))
    }
}

#[macro_export]
macro_rules! init {
    () => {
        rust_i18n::i18n!(backend = rs_localization::SharedBackend);
    };
}

pub use rust_i18n::set_locale;
pub use rust_i18n::t;
