use crate::layout::Layout;
use assets::files::avatar_svg;
use db::User;
use dioxus::prelude::*;

// Define the properties for IndexPage
#[derive(Props, Clone, PartialEq)] // Add Clone and PartialEq here
pub struct IndexPageProps {
    pub users: Vec<User>,
}

// Define the IndexPage component
#[component]
pub fn IndexPage(props: IndexPageProps) -> Element {
    rsx! {
        Layout { title: "Users Table",
            table {
                button { id: "alert-btn", "Click me!" }
                thead {
                    tr {
                        th { "ID" }
                        th { "Email" }
                    }
                }
                tbody {
                    for user in props.users {
                        tr {
                            td {
                                img {
                                    src: format!("/static/{}", avatar_svg.name),
                                    width: "16",
                                    height: "16"
                                }
                                strong { "{user.id}" }
                            }
                            td { "{user.email}" }
                        }
                    }
                }
            }
            script {
                r#type: "module",
                dangerous_inner_html: r#"
import init from '/static/components.js';
init();
"#
            }
        }
    }
}
