use crate::{
    checker::{analyze_with_variant, Category, EnglishVariant, Issue},
    storage::{Document, Store},
};
use dioxus::prelude::*;

struct HighlightSegment {
    text: String,
    issue: Option<usize>,
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Editor,
    Documents,
}

#[derive(Clone, Copy, PartialEq)]
enum SaveStatus {
    Saved,
    Saving,
    Error,
}

#[derive(Clone, Copy, PartialEq)]
enum ConnectionStatus {
    Idle,
    Checking,
    Connected,
    Error,
}

#[derive(Clone, Copy, PartialEq)]
enum AiStatus {
    Idle,
    Checking,
    Complete,
    Error,
}

fn highlight_segments(text: &str, issues: &[Issue]) -> Vec<HighlightSegment> {
    let mut segments = Vec::with_capacity(issues.len() * 2 + 1);
    let mut cursor = 0;

    for (index, issue) in issues.iter().enumerate() {
        if issue.start > cursor {
            segments.push(HighlightSegment {
                text: text[cursor..issue.start].to_owned(),
                issue: None,
            });
        }

        segments.push(HighlightSegment {
            text: text[issue.start..issue.end].to_owned(),
            issue: Some(index),
        });
        cursor = issue.end;
    }

    if cursor < text.len() {
        segments.push(HighlightSegment {
            text: text[cursor..].to_owned(),
            issue: None,
        });
    }

    segments
}

fn update_document(mut store: Signal<Store>, id: &str, update: impl FnOnce(&mut Document)) -> bool {
    let mut current = store();
    let Some(document) = current.documents.iter_mut().find(|doc| doc.id == id) else {
        return false;
    };

    update(document);
    document.updated_at = crate::storage::timestamp();
    current
        .documents
        .sort_by_key(|document| std::cmp::Reverse(document.updated_at));
    store.set(current);
    true
}

fn schedule_save(
    store: Signal<Store>,
    mut save_status: Signal<SaveStatus>,
    mut save_message: Signal<String>,
    mut save_revision: Signal<u64>,
) {
    let revision = save_revision() + 1;
    save_revision.set(revision);
    save_status.set(SaveStatus::Saving);
    save_message.set("Saving changes…".to_owned());

    spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(650)).await;
        if save_revision() != revision {
            return;
        }

        match crate::storage::save_store(&store()) {
            Ok(()) => {
                save_status.set(SaveStatus::Saved);
                save_message.set("Saved on this device".to_owned());
            }
            Err(error) => {
                save_status.set(SaveStatus::Error);
                save_message.set(error);
            }
        }
    });
}

fn save_now(
    store: Signal<Store>,
    mut save_status: Signal<SaveStatus>,
    mut save_message: Signal<String>,
    mut save_revision: Signal<u64>,
) {
    save_revision.set(save_revision() + 1);
    save_status.set(SaveStatus::Saving);

    match crate::storage::save_store(&store()) {
        Ok(()) => {
            save_status.set(SaveStatus::Saved);
            save_message.set("Saved on this device".to_owned());
        }
        Err(error) => {
            save_status.set(SaveStatus::Error);
            save_message.set(error);
        }
    }
}

fn document_count_label(count: usize) -> &'static str {
    if count == 1 {
        "1 document"
    } else {
        "documents"
    }
}

#[component]
pub fn Home() -> Element {
    let loaded_store = use_signal(crate::storage::load_store);
    let initial = loaded_store.read().clone();
    let initial_id = initial
        .store
        .documents
        .first()
        .map(|document| document.id.clone());
    let initial_status = if initial.error.is_some() {
        SaveStatus::Error
    } else {
        SaveStatus::Saved
    };
    let initial_message = initial
        .error
        .clone()
        .unwrap_or_else(|| "Saved on this device".to_owned());

    let mut store = use_signal(|| initial.store);
    let mut active_document_id = use_signal(|| initial_id);
    let mut delete_target = use_signal(|| None::<String>);
    let mut page = use_signal(|| Page::Editor);
    let mut active_issue = use_signal(|| Some(0_usize));
    let mut filter = use_signal(|| "All".to_owned());
    let mut settings_open = use_signal(|| false);
    let mut settings_saved = use_signal(|| false);
    let mut api_key = use_signal(String::new);
    let mut installed_models = use_signal(Vec::<String>::new);
    let mut connection_status = use_signal(|| ConnectionStatus::Idle);
    let mut connection_message = use_signal(String::new);
    let mut ai_issues = use_signal(Vec::<Issue>::new);
    let mut ai_status = use_signal(|| AiStatus::Idle);
    let mut ai_message = use_signal(String::new);
    let save_status = use_signal(|| initial_status);
    let save_message = use_signal(|| initial_message);
    let save_revision = use_signal(|| 0_u64);

    let current_store = store();
    let documents = current_store.documents.clone();
    let document_count = documents.len();
    let selected_id = active_document_id();
    let active_document = documents
        .iter()
        .find(|document| Some(&document.id) == selected_id.as_ref())
        .or_else(|| documents.first())
        .cloned();
    let current_text = active_document
        .as_ref()
        .map(|document| document.content.clone())
        .unwrap_or_default();
    let current_title = active_document
        .as_ref()
        .map(|document| document.title.clone())
        .unwrap_or_else(|| "Untitled document".to_owned());
    let active_id = active_document.as_ref().map(|document| document.id.clone());
    let current_settings = current_store.settings.clone();
    let english_variant = EnglishVariant::from_setting(&current_settings.english_variant);

    let local_issues = analyze_with_variant(&current_text, english_variant);
    let mut issues = local_issues.clone();
    for issue in ai_issues() {
        let overlaps_local = issues
            .iter()
            .any(|local| issue.start < local.end && local.start < issue.end);
        if !overlaps_local {
            issues.push(issue);
        }
    }
    issues.sort_by_key(|issue| issue.start);
    let preferred_index = active_issue()
        .filter(|index| *index < issues.len())
        .unwrap_or(0);
    let current_filter = filter();
    let grammar_count = issues
        .iter()
        .filter(|issue| issue.category == Category::Grammar)
        .count();
    let spelling_count = issues
        .iter()
        .filter(|issue| issue.category == Category::Spelling)
        .count();
    let filtered_issues = issues
        .iter()
        .enumerate()
        .filter(|(_, issue)| match current_filter.as_str() {
            "Grammar" => issue.category == Category::Grammar,
            "Spelling" => issue.category == Category::Spelling,
            _ => true,
        })
        .collect::<Vec<_>>();
    let selected_index = if filtered_issues
        .iter()
        .any(|(index, _)| *index == preferred_index)
    {
        preferred_index
    } else {
        filtered_issues
            .first()
            .map(|(index, _)| *index)
            .unwrap_or(preferred_index)
    };
    let word_count = current_text.split_whitespace().count();
    let character_count = current_text.chars().count();
    let selected = issues.get(selected_index);
    let score = 100_usize.saturating_sub(issues.len() * 4);
    let current_provider = current_settings.provider.clone();
    let delete_confirmation = delete_target().map(|target_id| {
        let target_title = documents
            .iter()
            .find(|document| document.id == target_id)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "this document".to_owned());
        (target_id, target_title)
    });
    let known_models = installed_models();
    let has_installed_models = !known_models.is_empty();
    let is_checking_connection = connection_status() == ConnectionStatus::Checking;
    let is_checking_ai = ai_status() == AiStatus::Checking;
    let is_saving = save_status() == SaveStatus::Saving;

    rsx! {
        main { class: "app-shell",
            aside { class: "left-rail",
                div { class: "brand",
                    span { class: "brand-mark", "gc" }
                    span { "Grammar Check" }
                }

                div { class: "rail-section-label", "YOUR WRITING" }
                nav { class: "primary-nav", aria_label: "Main navigation",
                    button {
                        class: if page() == Page::Editor { "nav-item active" } else { "nav-item" },
                        onclick: move |_| page.set(Page::Editor),
                        span { class: "nav-icon", "✎" }
                        "Editor"
                    }
                    button {
                        class: if page() == Page::Documents { "nav-item active" } else { "nav-item" },
                        onclick: move |_| page.set(Page::Documents),
                        span { class: "nav-icon", "▤" }
                        "Documents"
                        span { class: "nav-count", "{document_count}" }
                    }
                }

                div { class: "rail-bottom",
                    button {
                        class: "new-document-button",
                        onclick: move |_| {
                            let mut current = store();
                            let document = Document::new("Untitled document".to_owned());
                            let id = document.id.clone();
                            current.documents.insert(0, document);
                            store.set(current);
                            active_document_id.set(Some(id));
                            page.set(Page::Editor);
                            active_issue.set(None);
                            ai_issues.set(Vec::new());
                            ai_status.set(AiStatus::Idle);
                            ai_message.set(String::new());
                            schedule_save(store, save_status, save_message, save_revision);
                        },
                        span { "＋" }
                        "New document"
                    }
                }
            }

            section { class: "main-area",
                header { class: "topbar",
                    div { class: "breadcrumb",
                        button {
                            class: "breadcrumb-link",
                            onclick: move |_| page.set(Page::Documents),
                            "Documents"
                        }
                        if page() == Page::Editor {
                            span { class: "breadcrumb-divider", "/" }
                            strong { "{current_title}" }
                        }
                    }
                    div { class: "topbar-actions",
                        span {
                            class: if save_status() == SaveStatus::Error { "save-status save-error" } else { "save-status" },
                            title: "{save_message()}",
                            span {
                                class: if save_status() == SaveStatus::Error { "status-dot status-dot-error" } else if is_saving { "status-dot status-dot-saving" } else { "status-dot" }
                            }
                            match save_status() {
                                SaveStatus::Saved => "Saved on this device",
                                SaveStatus::Saving => "Saving…",
                                SaveStatus::Error => "Save failed",
                            }
                        }
                        button {
                            class: "button button-quiet",
                            onclick: move |_| {
                                settings_saved.set(false);
                                settings_open.set(true);
                            },
                            span { class: "button-icon", "⚙" }
                            "AI settings"
                        }
                    }
                }

                if page() == Page::Documents {
                    section { class: "documents-page",
                        div { class: "documents-page-heading",
                            div {
                                div { class: "eyebrow", "YOUR LIBRARY" }
                                h1 { "Documents" }
                                p { "Your drafts are saved privately on this device." }
                            }
                            button {
                                class: "button button-primary",
                                onclick: move |_| {
                                    let mut current = store();
                                    let document = Document::new("Untitled document".to_owned());
                                    let id = document.id.clone();
                                    current.documents.insert(0, document);
                                    store.set(current);
                                    active_document_id.set(Some(id));
                                    page.set(Page::Editor);
                                    active_issue.set(None);
                                    ai_issues.set(Vec::new());
                                    ai_status.set(AiStatus::Idle);
                                    ai_message.set(String::new());
                                    schedule_save(store, save_status, save_message, save_revision);
                                },
                                "＋ New document"
                            }
                        }
                        div { class: "documents-list-header",
                            span { "{document_count_label(document_count)}" }
                            span { "SAVED LOCALLY" }
                        }
                        if documents.is_empty() {
                            div { class: "documents-empty",
                                span { class: "empty-check", "▤" }
                                strong { "No documents yet" }
                                p { "Create a document to start writing and checking your grammar." }
                            }
                        } else {
                            div { class: "documents-list",
                                for document in documents.clone() {
                                    article { class: "document-list-item",
                                        button {
                                            class: "document-open",
                                            onclick: {
                                                let id = document.id.clone();
                                                move |_| {
                                                    active_document_id.set(Some(id.clone()));
                                                    active_issue.set(Some(0));
                                                    ai_issues.set(Vec::new());
                                                    ai_status.set(AiStatus::Idle);
                                                    ai_message.set(String::new());
                                                    page.set(Page::Editor);
                                                }
                                            },
                                            span { class: "document-list-icon", "▤" }
                                            span { class: "document-list-copy",
                                                strong { if document.title.trim().is_empty() { "Untitled document" } else { "{document.title}" } }
                                                small {
                                                    if document.content.trim().is_empty() {
                                                        "Empty document"
                                                    } else {
                                                        "{document.content.split_whitespace().count()} words · Saved on this device"
                                                    }
                                                }
                                            }
                                            span { class: "document-open-chevron", "›" }
                                        }
                                        button {
                                            class: "document-delete",
                                            title: "Delete document",
                                            aria_label: "Delete {document.title}",
                                            onclick: {
                                                let id = document.id.clone();
                                                move |_| delete_target.set(Some(id.clone()))
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                        if save_status() == SaveStatus::Error {
                            div { class: "storage-error-message", "{save_message()}" }
                        }
                    }
                } else if let Some(document_id) = active_id.clone() {
                    div { class: "workspace",
                        section { class: "editor-column",
                            div { class: "document-heading",
                                div {
                                    div { class: "eyebrow", "YOUR DOCUMENT" }
                                    input {
                                        class: "document-heading-input",
                                        value: current_title.clone(),
                                        aria_label: "Document name",
                                        oninput: {
                                            let document_id = document_id.clone();
                                            move |event| {
                                                let value = event.value();
                                                update_document(store, &document_id, |document| document.title = value);
                                                schedule_save(store, save_status, save_message, save_revision);
                                            }
                                        },
                                        onblur: move |_| save_now(store, save_status, save_message, save_revision),
                                    }
                                }
                                div { class: "document-actions",
                                    button {
                                        class: "button button-quiet",
                                        disabled: is_checking_ai || current_text.trim().is_empty(),
                                        title: "Send this document to your selected provider for an AI grammar check",
                                        onclick: {
                                            let document_text = current_text.clone();
                                            move |_| {
                                                let document_text = document_text.clone();
                                                async move {
                                                let settings = store().settings;
                                                let selected_provider = settings.provider.clone();
                                                ai_status.set(AiStatus::Checking);
                                                ai_message.set(format!("Checking with {selected_provider}…"));
                                                ai_issues.set(Vec::new());

                                                match crate::providers::check_text(
                                                    crate::providers::TextCheckRequest {
                                                        provider: &selected_provider,
                                                        endpoint: &settings.endpoint,
                                                        api_key: &api_key(),
                                                        model: &settings.model,
                                                        temperature: &settings.temperature,
                                                        thinking_enabled: settings.thinking_enabled,
                                                        english_variant: english_variant.label(),
                                                        text: &document_text,
                                                    },
                                                )
                                                .await
                                                {
                                                    Ok(suggestions) => {
                                                        let variant = EnglishVariant::from_setting(
                                                            &settings.english_variant,
                                                        );
                                                        let local = analyze_with_variant(&document_text, variant);
                                                        let suggestions = suggestions
                                                            .into_iter()
                                                            .filter(|suggestion| {
                                                                !local.iter().any(|issue| {
                                                                    suggestion.start < issue.end
                                                                        && issue.start < suggestion.end
                                                                })
                                                            })
                                                            .collect::<Vec<_>>();
                                                        let count = suggestions.len();
                                                        ai_issues.set(suggestions);
                                                        ai_status.set(AiStatus::Complete);
                                                        ai_message.set(format!(
                                                            "AI check complete. {count} additional suggestion(s)."
                                                        ));
                                                    }
                                                    Err(error) => {
                                                        ai_status.set(AiStatus::Error);
                                                        ai_message.set(error);
                                                    }
                                                }
                                                }
                                            }
                                        },
                                        if is_checking_ai { "Checking with AI…" } else { "Check with AI" }
                                    }
                                    button {
                                        class: "button button-primary",
                                        disabled: issues.is_empty(),
                                        onclick: {
                                            let document_id = document_id.clone();
                                            let corrections = issues.clone();
                                            move |_| {
                                                update_document(store, &document_id, |document| {
                                                    for issue in corrections.iter().rev() {
                                                        document.content.replace_range(issue.start..issue.end, &issue.suggestion);
                                                    }
                                                });
                                                active_issue.set(Some(0));
                                                ai_issues.set(Vec::new());
                                                ai_status.set(AiStatus::Idle);
                                                ai_message.set(String::new());
                                                schedule_save(store, save_status, save_message, save_revision);
                                            }
                                        },
                                        "Accept all"
                                    }
                                }
                            }

                            article { class: "editor-card",
                                div { class: "editor-toolbar",
                                    div { class: "toolbar-left",
                                        span { class: "document-icon", "▤" }
                                        span { "Draft" }
                                        span { class: "toolbar-divider" }
                                        select {
                                            class: "language-select",
                                            aria_label: "English spelling variant",
                                            value: current_settings.english_variant.clone(),
                                            onchange: move |event| {
                                                let mut current = store();
                                                current.settings.english_variant = event.value();
                                                store.set(current);
                                                ai_issues.set(Vec::new());
                                                ai_status.set(AiStatus::Idle);
                                                ai_message.set(String::new());
                                                schedule_save(store, save_status, save_message, save_revision);
                                            },
                                            option { value: "US English", "US English" }
                                            option { value: "UK English", "UK English" }
                                        }
                                    }
                                    div { class: "toolbar-right",
                                        span { class: "autosave-label", "{save_message()}" }
                                        button {
                                            class: "icon-button",
                                            title: "Save now",
                                            aria_label: "Save document now",
                                            onclick: move |_| save_now(store, save_status, save_message, save_revision),
                                            "↓"
                                        }
                                    }
                                }

                                div { class: "editor-paper",
                                    div { class: "writing-area",
                                        div { class: "line-numbers", aria_hidden: "true",
                                            for line in 1..=current_text.lines().count().max(1) {
                                                span { "{line:02}" }
                                            }
                                        }
                                        div { class: "text-layer",
                                            div { class: "highlight-layer", aria_hidden: "true",
                                                for segment in highlight_segments(&current_text, &issues) {
                                                    if let Some(index) = segment.issue {
                                                        mark {
                                                            class: if selected_index == index { "issue-mark active" } else { "issue-mark" },
                                                            onmouseenter: move |_| active_issue.set(Some(index)),
                                                            "{segment.text}"
                                                        }
                                                    } else {
                                                        span { "{segment.text}" }
                                                    }
                                                }
                                            }
                                            textarea {
                                                class: "writing-input",
                                                value: current_text.clone(),
                                                placeholder: "Start writing, or paste your text here…",
                                                spellcheck: "false",
                                                aria_label: "Document text",
                                                oninput: {
                                                    let document_id = document_id.clone();
                                                    move |event| {
                                                        let value = event.value();
                                                        update_document(store, &document_id, |document| document.content = value);
                                                        active_issue.set(Some(0));
                                                        ai_issues.set(Vec::new());
                                                        ai_status.set(AiStatus::Idle);
                                                        ai_message.set(String::new());
                                                        schedule_save(store, save_status, save_message, save_revision);
                                                    }
                                                },
                                                onblur: move |_| save_now(store, save_status, save_message, save_revision),
                                            }
                                        }
                                    }
                                    div { class: "paper-footer",
                                        span { "Basic offline check · not AI" }
                                        span { "{word_count} words" }
                                        span { "{character_count} characters" }
                                    }
                                }

                                div { class: "editor-note",
                                    span { class: "sparkle", "✦" }
                                    "Offline = built-in common-error patterns, not a full dictionary or AI. Check with AI for contextual review."
                                }
                                if ai_status() != AiStatus::Idle {
                                    div {
                                        class: match ai_status() {
                                            AiStatus::Complete => "ai-result ai-result-complete",
                                            AiStatus::Error => "ai-result ai-result-error",
                                            AiStatus::Checking => "ai-result ai-result-checking",
                                            AiStatus::Idle => "ai-result",
                                        },
                                        "{ai_message()}"
                                    }
                                }
                            }

                            div { class: "quality-strip",
                                div { class: "quality-score",
                                    span { class: "quality-ring", "{score}" }
                                    div {
                                        strong { if issues.is_empty() { "No rule matches" } else { "Rule check complete" } }
                                        span { "This score reflects the built-in rules only." }
                                    }
                                }
                                div { class: "quality-divider" }
                                div { class: "quality-stat",
                                    strong { "{issues.len()}" }
                                    span { "suggestions" }
                                }
                                div { class: "quality-stat",
                                    strong { "{word_count}" }
                                    span { "words" }
                                }
                            }
                        }

                        aside { class: "review-panel",
                            div { class: "review-header",
                                div {
                                    div { class: "eyebrow",
                                        if ai_status() == AiStatus::Complete { "AI + COMMON RULES" } else { "COMMON RULES · OFFLINE" }
                                    }
                                    h2 { "Writing review" }
                                }
                                span { class: "issue-total", "{issues.len()}" }
                            }

                            div { class: "review-summary",
                                div { class: "summary-topline",
                                    span { "Suggestions" }
                                    span { class: "summary-count", "{issues.len()} to review" }
                                }
                                div { class: "summary-categories",
                                    button {
                                        class: if filter() == "All" { "summary-category selected" } else { "summary-category" },
                                        onclick: move |_| filter.set("All".to_owned()),
                                        span { class: "category-dot grammar-dot" }
                                        "All"
                                        strong { "{issues.len()}" }
                                    }
                                    button {
                                        class: if filter() == "Grammar" { "summary-category selected" } else { "summary-category" },
                                        onclick: move |_| filter.set("Grammar".to_owned()),
                                        span { class: "category-dot grammar-dot" }
                                        "Grammar"
                                        strong { "{grammar_count}" }
                                    }
                                    button {
                                        class: if filter() == "Spelling" { "summary-category selected" } else { "summary-category" },
                                        onclick: move |_| filter.set("Spelling".to_owned()),
                                        span { class: "category-dot spelling-dot" }
                                        "Spelling"
                                        strong { "{spelling_count}" }
                                    }
                                }
                            }

                            if issues.is_empty() {
                                div { class: "empty-review",
                                    span { class: "empty-check", "✓" }
                                    strong { "No built-in rule matches" }
                                    p { "This offline check catches common patterns, not every grammar or spelling error." }
                                }
                            } else {
                                div { class: "issue-list",
                                    for (index, issue) in filtered_issues {
                                        button {
                                            class: if selected_index == index { "issue-item selected" } else { "issue-item" },
                                            onclick: move |_| active_issue.set(Some(index)),
                                            onmouseenter: move |_| active_issue.set(Some(index)),
                                            div { class: "issue-item-top",
                                                span {
                                                    class: if issue.category == Category::Grammar { "issue-type grammar-type" } else { "issue-type spelling-type" },
                                                    "{issue.category.label()}"
                                                }
                                                span { class: "issue-chevron", "›" }
                                            }
                                            div { class: "issue-original", "{issue.incorrect}" }
                                            div { class: "issue-suggestion",
                                                span { class: "suggestion-arrow", "↳" }
                                                "{issue.suggestion}"
                                            }
                                        }
                                    }
                                }

                                if let Some(issue) = selected {
                                    div { class: "learning-card",
                                        div { class: "learning-heading",
                                            span { class: "learning-icon", "i" }
                                            span { "THE RULE" }
                                        }
                                        strong { "{issue.rule}" }
                                        p { "{issue.explanation}" }
                                        button {
                                            class: "apply-suggestion",
                                            onclick: {
                                                let issue = issue.clone();
                                                let document_id = document_id.clone();
                                                move |_| {
                                                    update_document(store, &document_id, |document| {
                                                        document.content.replace_range(issue.start..issue.end, &issue.suggestion);
                                                    });
                                                    active_issue.set(Some(0));
                                                    schedule_save(store, save_status, save_message, save_revision);
                                                }
                                            },
                                            "Accept suggestion"
                                            span { "→" }
                                        }
                                    }
                                }
                            }

                            div { class: "review-footer",
                                span { class: "local-indicator" }
                                "Common spelling + grammar patterns · {english_variant.label()}"
                            }
                        }
                    }
                } else {
                    section { class: "documents-empty no-active-document",
                        span { class: "empty-check", "▤" }
                        strong { "No document selected" }
                        p { "Create a document to start writing." }
                        button {
                            class: "button button-primary",
                            onclick: move |_| {
                                let mut current = store();
                                let document = Document::new("Untitled document".to_owned());
                                let id = document.id.clone();
                                current.documents.insert(0, document);
                                store.set(current);
                                active_document_id.set(Some(id));
                                schedule_save(store, save_status, save_message, save_revision);
                            },
                            "＋ New document"
                        }
                    }
                }
            }

            if settings_open() {
                div { class: "modal-backdrop",
                    section { class: "settings-modal", role: "dialog", aria_modal: "true", aria_labelledby: "settings-title",
                        header { class: "modal-header",
                            div {
                                div { class: "eyebrow", "PREFERENCES" }
                                h2 { id: "settings-title", "AI provider settings" }
                                p { "Choose a provider and tune its writing assistant." }
                            }
                            button {
                                class: "icon-button modal-close",
                                aria_label: "Close settings",
                                onclick: move |_| settings_open.set(false),
                                "×"
                            }
                        }
                        div { class: "provider-options",
                            for name in ["Ollama", "OpenAI", "Anthropic", "Google"] {
                                button {
                                    class: if current_provider == name { "provider-option active" } else { "provider-option" },
                                    disabled: is_checking_connection,
                                    onclick: move |_| {
                                        let mut current = store();
                                        current.settings.provider = name.to_owned();
                                        current.settings.model = match name {
                                            "Ollama" => "qwen2.5:3b",
                                            "OpenAI" => "gpt-4.1-mini",
                                            "Anthropic" => "claude-3-5-haiku-latest",
                                            _ => "gemini-2.0-flash",
                                        }.to_owned();
                                        current.settings.endpoint = if name == "Ollama" {
                                            "http://localhost:11434".to_owned()
                                        } else {
                                            String::new()
                                        };
                                        if name == "Ollama" {
                                            current.settings.temperature = "0.1".to_owned();
                                            current.settings.thinking_enabled = false;
                                        }
                                        store.set(current);
                                        api_key.set(String::new());
                                        installed_models.set(Vec::new());
                                        connection_status.set(ConnectionStatus::Idle);
                                        connection_message.set(String::new());
                                        schedule_save(store, save_status, save_message, save_revision);
                                    },
                                    span { class: "provider-radio" }
                                    "{name}"
                                    if name == "Ollama" {
                                        span { class: "local-badge", "LOCAL" }
                                    }
                                }

                                if let Some((target_id, target_title)) = delete_confirmation.clone() {
                                    div { class: "modal-backdrop",
                                        section { class: "settings-modal delete-modal", role: "dialog", aria_modal: "true", aria_labelledby: "delete-title",
                                            div {
                                                div { class: "eyebrow", "DELETE DOCUMENT" }
                                                h2 { id: "delete-title", "Delete this document?" }
                                                p { "“{target_title}” will be permanently removed from this device." }
                                            }
                                            footer { class: "modal-footer",
                                                button {
                                                    class: "button button-quiet",
                                                    onclick: move |_| delete_target.set(None),
                                                    "Cancel"
                                                }
                                                button {
                                                    class: "button button-danger",
                                                    onclick: {
                                                        let target_id = target_id.clone();
                                                        move |_| {
                                                            let mut current = store();
                                                            current.documents.retain(|document| document.id != target_id);
                                                            let next_active = current.documents.first().map(|document| document.id.clone());
                                                            store.set(current);
                                                            if active_document_id() == Some(target_id.clone()) {
                                                                active_document_id.set(next_active);
                                                                ai_issues.set(Vec::new());
                                                                ai_status.set(AiStatus::Idle);
                                                                ai_message.set(String::new());
                                                            }
                                                            delete_target.set(None);
                                                            schedule_save(store, save_status, save_message, save_revision);
                                                        }
                                                    },
                                                    "Delete permanently"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if current_provider == "Ollama" {
                            div { class: "recommendation-banner",
                                span { class: "sparkle", "✦" }
                                div {
                                    strong { "Recommended for smaller local models" }
                                    p { "Qwen 2.5 3B · low temperature · thinking off for faster, structured edits." }
                                }
                            }
                        }
                        label { class: "field-label", "Model",
                            input {
                                class: "settings-input",
                                value: current_settings.model.clone(),
                                placeholder: if current_provider == "Ollama" { "e.g. qwen2.5:3b" } else { "Model ID" },
                                oninput: move |event| {
                                    let mut current = store();
                                    current.settings.model = event.value();
                                    store.set(current);
                                    schedule_save(store, save_status, save_message, save_revision);
                                },
                            }
                        }
                        if has_installed_models {
                            label { class: "field-label", if current_provider == "Ollama" { "Installed Ollama models" } else { "Available models" },
                                    select {
                                        class: "settings-input",
                                        value: current_settings.model.clone(),
                                        onchange: move |event| {
                                            let mut current = store();
                                            current.settings.model = event.value();
                                            store.set(current);
                                            schedule_save(store, save_status, save_message, save_revision);
                                        },
                                        for installed_model in known_models.clone() {
                                            option { value: "{installed_model}", "{installed_model}" }
                                        }
                                    }
                                }
                        } else if current_provider == "Ollama" {
                            p { class: "field-hint", "Check the connection to list models already installed in Ollama." }
                        }
                        label { class: "field-label", if current_provider == "Ollama" { "Ollama URL" } else { "API endpoint (optional)" },
                            input {
                                class: "settings-input",
                                value: current_settings.endpoint.clone(),
                                placeholder: if current_provider == "Ollama" { "http://localhost:11434" } else { "Use provider default" },
                                oninput: move |event| {
                                    let mut current = store();
                                    current.settings.endpoint = event.value();
                                    store.set(current);
                                    schedule_save(store, save_status, save_message, save_revision);
                                },
                            }
                        }
                        if current_provider != "Ollama" {
                            label { class: "field-label", "API key",
                                input {
                                    class: "settings-input",
                                    r#type: "password",
                                    value: api_key(),
                                    placeholder: "Enter your provider API key",
                                    autocomplete: "off",
                                    oninput: move |event| api_key.set(event.value()),
                                }
                            }
                        }
                        div { class: if current_provider == "Ollama" { "settings-row" } else { "settings-row single-setting" },
                            label { class: "field-label temperature-field", "Temperature",
                                div { class: "range-row",
                                    input {
                                        r#type: "range",
                                        min: "0",
                                        max: "1",
                                        step: "0.1",
                                        value: current_settings.temperature.clone(),
                                        oninput: move |event| {
                                            let mut current = store();
                                            current.settings.temperature = event.value();
                                            store.set(current);
                                            schedule_save(store, save_status, save_message, save_revision);
                                        },
                                    }
                                    span { class: "range-value", "{current_settings.temperature}" }
                                }
                            }
                            if current_provider == "Ollama" {
                                label { class: "thinking-toggle",
                                    span {
                                        strong { "Thinking mode" }
                                        small { "Uses more time and tokens" }
                                    }
                                    input {
                                        r#type: "checkbox",
                                        checked: current_settings.thinking_enabled,
                                        onchange: move |event| {
                                            let mut current = store();
                                            current.settings.thinking_enabled = event.checked();
                                            store.set(current);
                                            schedule_save(store, save_status, save_message, save_revision);
                                        },
                                    }
                                }
                            }
                        }
                        div { class: "settings-notice",
                            "Connection checks list available models. Grammar suggestions currently use built-in offline rules, not LLM requests."
                        }
                        div { class: "connection-controls",
                            button {
                                class: "button button-quiet check-connection-button",
                                disabled: is_checking_connection,
                                onclick: move |_| async move {
                                    let selected_settings = store().settings;
                                    let selected_provider = selected_settings.provider;
                                    let selected_endpoint = selected_settings.endpoint;
                                    let selected_api_key = api_key();
                                    connection_status.set(ConnectionStatus::Checking);
                                    connection_message.set(format!("Checking {selected_provider}…"));

                                    match crate::providers::check_connection(
                                        &selected_provider,
                                        &selected_endpoint,
                                        &selected_api_key,
                                    )
                                    .await
                                    {
                                        Ok(models) => {
                                            let model_count = models.len();
                                            if selected_provider == "Ollama" {
                                                let selected_model = store().settings.model;
                                                if !models.iter().any(|name| name == &selected_model) {
                                                    if let Some(first_model) = models.first() {
                                                        let mut current = store();
                                                        current.settings.model = first_model.clone();
                                                        store.set(current);
                                                        schedule_save(store, save_status, save_message, save_revision);
                                                    }
                                                }
                                                installed_models.set(models);
                                                connection_message.set(if model_count == 0 {
                                                    "Connected to Ollama, but no models are installed. Run `ollama pull <model>` to add one.".to_owned()
                                                } else {
                                                    format!("Connected to Ollama. Found {model_count} installed model(s).")
                                                });
                                            } else {
                                                installed_models.set(models);
                                                connection_message.set(format!(
                                                    "Connected to {selected_provider}. {model_count} model(s) available."
                                                ));
                                            }
                                            connection_status.set(ConnectionStatus::Connected);
                                        }
                                        Err(error) => {
                                            if selected_provider == "Ollama" {
                                                installed_models.set(Vec::new());
                                            }
                                            connection_message.set(error);
                                            connection_status.set(ConnectionStatus::Error);
                                        }
                                    }
                                },
                                if is_checking_connection { "Checking…" } else { "Check connection" }
                            }
                            if !connection_message().is_empty() {
                                div {
                                    class: match connection_status() {
                                        ConnectionStatus::Connected => "connection-result connected",
                                        ConnectionStatus::Error => "connection-result error",
                                        ConnectionStatus::Checking => "connection-result checking",
                                        ConnectionStatus::Idle => "connection-result",
                                    },
                                    "{connection_message}"
                                }
                            }
                        }
                        footer { class: "modal-footer",
                            if settings_saved() {
                                span { class: "saved-message", "Preferences saved on this device." }
                            }
                            button {
                                class: "button button-quiet",
                                onclick: move |_| settings_open.set(false),
                                "Close"
                            }
                            button {
                                class: "button button-primary",
                                onclick: move |_| {
                                    save_now(store, save_status, save_message, save_revision);
                                    settings_saved.set(true);
                                },
                                "Save preferences"
                            }
                        }
                    }
                }
            }
        }
    }
}
