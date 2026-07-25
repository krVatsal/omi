use dioxus::prelude::*;

use crate::app::Db;
use crate::recording::{LiveTranscript, RecordingStatus};
use omi_db::schema::{Conversation, Segment};

#[component]
pub fn ConversationsPage() -> Element {
    let db: Signal<Option<Db>> = use_context();
    let recording_status: Signal<RecordingStatus> = use_context();
    let live_transcript: Signal<LiveTranscript> = use_context();

    let mut past: Signal<Vec<Conversation>> = use_signal(Vec::new);
    let mut expanded: Signal<Option<String>> = use_signal(|| None);
    let mut expanded_segs: Signal<Vec<Segment>> = use_signal(Vec::new);
    let mut search_query = use_signal(String::new);
    let mut editing_id: Signal<Option<String>> = use_signal(|| None);
    let mut edit_title = use_signal(String::new);

    let is_recording = matches!(*recording_status.read(), RecordingStatus::Recording { .. });
    let live_segments = live_transcript.read().segments.clone();

    let mut reload = move || {
        if let Some(Db(ref d)) = *db.read() {
            let q = search_query.read().trim().to_string();
            let result = if q.is_empty() {
                d.list_conversations(50)
            } else {
                d.search_conversations(&q, 50)
            };
            match result {
                Ok(convs) => past.set(convs),
                Err(e) => tracing::error!("[CONVERSATIONS] Failed to load: {e}"),
            }
        }
    };

    let db_for_load = db.clone();
    use_effect(move || {
        let _db_snap = db_for_load.read().clone();
        let is_rec = matches!(*recording_status.read(), RecordingStatus::Recording { .. });
        if !is_rec {
            if let Some(Db(d)) = _db_snap {
                match d.list_conversations(50) {
                    Ok(convs) => past.set(convs),
                    Err(e) => tracing::error!("[CONVERSATIONS] Failed to load: {e}"),
                }
            }
        }
    });

    rsx! {
        div { class: "page",
            h1 { class: "page-title", "Conversations" }

            div { class: "search-bar",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Search conversations...",
                    value: "{search_query}",
                    oninput: move |e| {
                        search_query.set(e.value());
                        reload();
                    },
                }
            }

            if is_recording {
                div { class: "conversation-card live-card",
                    div { class: "conversation-header",
                        h3 { "● Live Session" }
                        span { class: "conversation-meta",
                            "{live_segments.iter().filter(|s| s.is_final).count()} segments"
                        }
                    }
                    div { class: "conversation-transcript",
                        for seg in live_segments.iter().filter(|s| s.is_final) {
                            div { class: "transcript-segment final",
                                span { class: "speaker-badge", "S{seg.speaker}" }
                                span { class: "segment-text", "{seg.text}" }
                            }
                        }
                        if let Some(interim) = live_segments.iter().rfind(|s| !s.is_final) {
                            div { class: "transcript-segment interim",
                                span { class: "speaker-badge muted", "S{interim.speaker}" }
                                span { class: "segment-text muted", "{interim.text}" }
                            }
                        }
                    }
                }
            }

            if past.read().is_empty() && !is_recording {
                div { class: "empty-state",
                    if search_query.read().is_empty() {
                        p { "No conversations recorded yet." }
                        p { class: "text-muted", "Go to Dashboard and start recording." }
                    } else {
                        p { "No conversations matching your search." }
                    }
                }
            }

            for conv in past.read().iter() {
                {
                    let conv_id = conv.id.clone();
                    let conv_id_del = conv.id.clone();
                    let conv_id_edit = conv.id.clone();
                    let conv_id_edit2 = conv.id.clone();
                    let is_expanded = expanded.read().as_deref() == Some(&conv.id);
                    let is_editing = editing_id.read().as_deref() == Some(&conv.id);
                    let title = conv.title.clone().unwrap_or_else(|| "Untitled".into());
                    let summary = conv.summary.clone().unwrap_or_default();
                    let duration = conv.duration_secs;
                    let started = conv.started_at.format("%b %d, %H:%M").to_string();
                    let status = conv.status.clone();
                    let db2 = db.clone();
                    let db_del = db.clone();
                    let db_edit = db.clone();

                    rsx! {
                        div {
                            key: "{conv_id}",
                            class: if is_expanded { "conversation-card expanded" } else { "conversation-card" },
                            onclick: move |_| {
                                if is_editing { return; }
                                if is_expanded {
                                    expanded.set(None);
                                } else {
                                    if let Some(Db(ref d)) = *db2.read() {
                                        match d.get_segments(&conv_id) {
                                            Ok(segs) => {
                                                expanded_segs.set(segs);
                                                expanded.set(Some(conv_id.clone()));
                                            }
                                            Err(e) => tracing::error!("[CONVERSATIONS] Failed to load segments: {e}"),
                                        }
                                    }
                                }
                            },
                            div { class: "conversation-header",
                                if is_editing {
                                    input {
                                        class: "settings-input",
                                        r#type: "text",
                                        value: "{edit_title}",
                                        onclick: move |e| e.stop_propagation(),
                                        oninput: move |e| edit_title.set(e.value()),
                                        onkeydown: move |e| {
                                            if e.key() == Key::Enter {
                                                e.prevent_default();
                                                if let Some(Db(ref d)) = *db_edit.read() {
                                                    let _ = d.update_conversation_title(&conv_id_edit, &edit_title.read());
                                                }
                                                editing_id.set(None);
                                                reload();
                                            } else if e.key() == Key::Escape {
                                                editing_id.set(None);
                                            }
                                        },
                                    }
                                } else {
                                    h3 { "{title}" }
                                }
                                div { class: "conversation-meta-row",
                                    span { class: "conversation-meta", "{started}" }
                                    span { class: "conversation-meta", "{duration:.0}s" }
                                    span {
                                        class: if status == "completed" { "status-badge completed" } else { "status-badge" },
                                        "{status}"
                                    }
                                    button {
                                        class: "btn-icon",
                                        title: "Edit title",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            edit_title.set(title.clone());
                                            editing_id.set(Some(conv_id_edit2.clone()));
                                        },
                                        "✎"
                                    }
                                    button {
                                        class: "btn-icon btn-danger",
                                        title: "Delete conversation",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            if let Some(Db(ref d)) = *db_del.read() {
                                                let _ = d.delete_conversation(&conv_id_del);
                                            }
                                            reload();
                                        },
                                        "✕"
                                    }
                                }
                            }
                            if !summary.is_empty() {
                                p { class: "conversation-summary", "{summary}" }
                            }
                            if is_expanded {
                                div { class: "conversation-transcript",
                                    for seg in expanded_segs.read().iter() {
                                        div { class: "transcript-segment final",
                                            span { class: "speaker-badge", "S{seg.speaker}" }
                                            span { class: "segment-time", "{seg.start_time:.1}s" }
                                            span { class: "segment-text", "{seg.text}" }
                                        }
                                    }
                                    if expanded_segs.read().is_empty() {
                                        p { class: "text-muted", "No segments stored." }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
