use rust_i18n::Backend;
use std::borrow::Cow;

rust_i18n::i18n!("../../Resource/locales");

pub struct SharedBackend;

impl Backend for SharedBackend {
    fn available_locales(&self) -> Vec<Cow<'_, str>> {
        _RUST_I18N_BACKEND.available_locales()
    }

    fn translate(&self, locale: &str, key: &str) -> Option<Cow<'_, str>> {
        _RUST_I18N_BACKEND
            .translate(locale, key)
            .or_else(|| _RUST_I18N_BACKEND.translate("en", key))
    }

    fn messages_for_locale(&self, locale: &str) -> Option<Vec<(Cow<'_, str>, Cow<'_, str>)>> {
        _RUST_I18N_BACKEND.messages_for_locale(locale)
    }
}

pub fn available_locales() -> Vec<Cow<'static, str>> {
    SharedBackend.available_locales()
}

#[macro_export]
macro_rules! init {
    () => {
        rust_i18n::i18n!(backend = rs_localization::SharedBackend);
    };
}

pub use rust_i18n::set_locale;
pub use rust_i18n::t;
