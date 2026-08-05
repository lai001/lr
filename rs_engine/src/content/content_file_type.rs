use rs_content::TypedContent;
use std::collections::HashMap;

pub type EContentFileType = rs_foundation::new::SingleThreadMutType<Box<dyn rs_content::Content>>;

pub fn find_content_by_type_ref<'a, 'b, T: rs_content::Content>(
    iter: impl IntoIterator<Item = &'a EContentFileType> + Clone,
    url: &'b url::Url,
) -> Option<std::cell::Ref<'a, T>> {
    for file in iter.into_iter() {
        let content = file.borrow();
        if content.get_url() == *url && content.as_ref().as_any().is::<T>() {
            let found = std::cell::Ref::filter_map(content, |content| {
                content.as_ref().as_any().downcast_ref::<T>()
            });
            return found.ok();
        }
    }
    return None;
}

pub fn find_content_by_type_mut<'a, 'b, T: rs_content::Content>(
    iter: impl IntoIterator<Item = &'a mut EContentFileType> + Clone,
    url: &'b url::Url,
) -> Option<std::cell::RefMut<'a, T>> {
    for file in iter.into_iter() {
        let content = file.borrow_mut();
        if content.get_url() == *url && content.as_ref().as_any().is::<T>() {
            let found = std::cell::RefMut::filter_map(content, |content| {
                content.as_mut().as_any_mut().downcast_mut::<T>()
            });
            return found.ok();
        }
    }
    return None;
}

pub fn find_content_by_type<'a, 'b, T: rs_content::Content>(
    iter: impl IntoIterator<Item = &'a EContentFileType> + Clone,
    url: &'b url::Url,
) -> Option<EContentFileType> {
    for file in iter.into_iter() {
        let content = file.borrow();
        if content.get_url() == *url && content.as_ref().as_any().is::<T>() {
            return Some(file.clone());
        }
    }
    return None;
}

pub fn collect_typed_contents<'a, T: rs_content::Content>(
    contents: impl IntoIterator<Item = &'a EContentFileType> + Clone,
) -> Vec<TypedContent<T>> {
    let mut typed_contents = vec![];
    for content in contents {
        let typed_content = TypedContent::<T>::new(content.clone());
        if let Ok(typed_content) = typed_content {
            typed_contents.push(typed_content);
        }
    }
    typed_contents
}

pub fn find_content_by_type_ref_map<'a, 'b, T: rs_content::Content>(
    contents: &'a HashMap<url::Url, EContentFileType>,
    url: &'b url::Url,
) -> Option<std::cell::Ref<'a, T>> {
    for (content_url, file) in contents {
        if content_url == url {
            let content = file.borrow();
            if content.as_ref().as_any().is::<T>() {
                let found = std::cell::Ref::filter_map(content, |content| {
                    content.as_ref().as_any().downcast_ref::<T>()
                });
                return found.ok();
            }
        }
    }
    return None;
}

pub fn find_content_by_type_mut_map<'a, 'b, T: rs_content::Content>(
    contents: &'a HashMap<url::Url, EContentFileType>,
    url: &'b url::Url,
) -> Option<std::cell::RefMut<'a, T>> {
    for (content_url, file) in contents {
        if content_url == url {
            let content = file.borrow_mut();
            if content.as_ref().as_any().is::<T>() {
                let found = std::cell::RefMut::filter_map(content, |content| {
                    content.as_mut().as_any_mut().downcast_mut::<T>()
                });
                return found.ok();
            }
        }
    }
    return None;
}

pub fn find_content_by_type_map<'a, 'b, T: rs_content::Content>(
    contents: &'a HashMap<url::Url, EContentFileType>,
    url: &'b url::Url,
) -> Option<EContentFileType> {
    for (content_url, file) in contents {
        if content_url == url {
            let content = file.borrow();
            if content.as_ref().as_any().is::<T>() {
                return Some(file.clone());
            }
        }
    }
    return None;
}

pub fn collect_typed_contents_map<'a, T: rs_content::Content>(
    contents: &'a HashMap<url::Url, EContentFileType>,
) -> Vec<TypedContent<T>> {
    let mut typed_contents = vec![];
    for (_, content) in contents {
        let typed_content = TypedContent::<T>::new(content.clone());
        if let Ok(typed_content) = typed_content {
            typed_contents.push(typed_content);
        }
    }
    typed_contents
}
