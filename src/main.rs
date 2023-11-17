#![allow(non_snake_case)]
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};

#[derive(PartialEq)]
enum FilterState {
    All,
    Active,
    Completed,
}

type Todos = im_rc::HashMap<u32, TodoItem>;

#[derive(Debug, PartialEq, Clone)]
struct TodoItem {
    id: u32,
    checked: bool,
    contents: String,
}

fn main() {
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    let todos = use_ref(cx, im_rc::HashMap::<u32, TodoItem>::default);
    let filter = use_state(cx, || FilterState::All);
    let draft = use_ref(cx, String::new);
    let todo_id = use_state(cx, || 0);

    let mut filtered_todos = todos
        .read()
        .iter()
        .filter(|(_, item)| match filter.get() {
            FilterState::All => true,
            FilterState::Active => !item.checked,
            FilterState::Completed => item.checked,
        })
        .map(|f| *f.0)
        .collect::<Vec<_>>();
    filtered_todos.sort_unstable();

    let show_clear_completed = todos.read().values().any(|todo| todo.checked);
    let items_left = filtered_todos.len();
    let items_text = match items_left {
        1 => "item",
        _ => "items",
    };

    cx.render(rsx! {
        section {
            header {
                h1 {"todos"}
                input {
                    placeholder: "What needs to be done?",
                    value: "{draft.read()}",
                    autofocus: "true",
                    oninput: move |evt| draft.set(evt.value.clone()),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter && !draft.read().is_empty() {
                            todos.write().insert(
                                *todo_id.get(),
                                TodoItem {
                                    id: *todo_id.get(),
                                    checked: false,
                                    contents: draft.read().clone(),
                                },
                            );
                            todo_id.set(todo_id + 1);
                            draft.set("".to_string());
                        }
                    }
                }
            }
            ul {
                filtered_todos.iter().map(|id| rsx!(
                    TodoEntry{ key: "{id}", id: *id, set_todos: todos }
                ))
            }
            if !todos.read().is_empty() {
                rsx! {
                    section {
                        span {
                            strong {"{items_left}"}
                            span {"{items_text} left"}
                        }
                        ul {
                            li {
                                a { onclick: move |_| filter.set(FilterState::All), "All" }
                                a { onclick: move |_| filter.set(FilterState::Active), "Active" }
                                a { onclick: move |_| filter.set(FilterState::Completed), "Completed" }
                            }
                        }
                        if show_clear_completed {
                            rsx!(
                                button {
                                    onclick: move |_| todos.write().retain(|_, todo| !todo.checked),
                                    "Clear completed"
                                }
                            )
                        }
                    }
                }
            }
            footer {
                p { "Double-click to edit a todo" }
            }
        }
    })
}

#[derive(Props)]
struct TodoEntryProps<'a> {
    set_todos: &'a UseRef<Todos>,
    id: u32,
}

fn TodoEntry<'a>(cx: Scope<'a, TodoEntryProps<'a>>) -> Element {
    let editing = use_state(cx, || false);
    let todos = cx.props.set_todos.read();
    let todo = &todos[&cx.props.id];
    let is_checked = todo.checked;
    let completed = if is_checked { "competed" } else { "" };
    let is_editing = (**editing).then_some("editing").unwrap_or("");

    render!(li {
        onclick: move |_| {
            if !is_checked {
                editing.set(true)
            }
        },
        onfocusout: move |_| editing.set(false),
        div {
            input {
                r#type: "checkbox",
                id: "todo-{todo.id}",
                checked: "{is_checked}",
                onchange: move |evt| {
                    cx.props.set_todos.write()[&cx.props.id].checked = evt.value.parse().unwrap();
                }
            }
            label { r#for: "todo-{todo.id}", pointer_events: "none", "{todo.contents}" }
        }
        if **editing {
            rsx!{
                input {
                    value: "{todo.contents}",
                    oninput: move |evt| cx.props.set_todos.write()[&cx.props.id].contents = evt.value.clone(),
                    autofocus: "true",
                    onkeydown: move |evt| {
                        match evt.key().to_string().as_str() {
                            "Enter" | "Escape" | "Tab" => editing.set(false),
                            _ => {}
                        }
                    },
                }
            }
        }
    })
}
