use std::env;
use std::{fs::OpenOptions, io::prelude::Write, path::Path};

use std::rc::Rc;
use std::process::{Command, Stdio};



use dioxus::prelude::*;

use crate::parser::{ast_as_d2_spec, parse_query};

pub fn App(cx: Scope) -> Element {
    render! {
        div {
            class: "min-h-full flex flex-col",
            div {
                class: "max-h-screen h-screen",
                div {
                    class: "flex h-full",
                    div {
                        class: "flex flex-col flex-1 w-full overflow-x-hidden",
                        NavBar {},
                        MainContent {},
                    }
                }
            }
        }
    }
}

pub fn NavBar(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "flex h-16 max-h-16 items-center justify-between py-2 px-5 border-b border-[#E6E8EB]",
            div {
                class: "-ml-2 flex items-center font-light text-sm",
                span {
                    class: "font-medium",
                    "DB to ERD",
                }
            },
            div {
                class: "flex items-center font-light",
                span {
                    class: "justify-center cursor-pointer items-center space-x-2 text-center rounded-md outline-none outline-0 focus-visible:outline-4 focus-visible:outline-offset-1 border text-white bg-[#2f2640] text-xs px-2.5 py-1 hidden md:flex",
                    "Help?"
                }
            }
        }
    })
}

// todo: clean up this component
pub fn MainContent(cx: Scope) -> Element {
    let query = use_state(cx, String::new);
    let count = use_state(cx, || String::new());

    let mut d2_ast: Vec<String> = vec![];

    if !query.is_empty() {
        let result = parse_query(query);

        let ast_str = match result {
            crate::parser::QueryAst::Parsed(stmts) => {
                stmts.into_iter().map(|s| {
                    ast_as_d2_spec(s)
                }).collect::<Vec<_>>().into_iter().map(|p| p.to_string()).collect::<Vec<_>>().join("\n")
            },
            crate::parser::QueryAst::InvalidSQL(err_msg) => err_msg,
        };

        d2_ast.push(ast_str)
    }

    let mut img_sources = vec![];
    if !d2_ast.is_empty() {
        for item in d2_ast {
            img_sources.push(generate_d2(item))
        }
    }

    cx.render(rsx! {
        div {
            class: "flex flex-row gap-2 h-full p-2",
            div {
                class: "min-w-[50vw] min-h-full p-2",
                div {
                    class: "min-w-full mb-4 border border-[#E6E8EB] rounded-lg bg-white",
                    div {
                        class: "flex items-center justify-between px-2 py-1 border-b",
                        div {},
                        button {
                            class: "inline-flex items-center py-2 px-2.5 text-xs font-medium text-center text-white bg-[#5746af] rounded-lg focus:ring-4 focus:ring-blue-200",
                            prevent_default: "onclick",
                            onclick: move |_| {
                                if count.is_empty() {
                                    count.set(parse_query(query).to_string())
                                } else {
                                    count.set("".to_string())
                                }
                            },
                            play_icon{}"Compile"
                        }
                    },
                    div {
                        class: "min-w-full px-4 py-2 bg-[#fafafa] rounded-b-lg",
                        form {
                            textarea {
                                class: "min-w-full min-h-[85vh] font-mono bg-[#fafafa] text-sm text-[#1a1523] px-1 bg-white border-0 focus:ring-0",
                                resize: "none",
                                placeholder: "Type something here...",
                                value: "{query}",
                                oninput: move |evt| query.set(evt.value.clone()),
                            }
                        }
                    }
                }
            },
            div {
                class: "min-w-[48vw] min-h-full p-2",
                div {
                    ul {
                        class: "flex flex-wrap text-sm font-medium text-center text-gray-500 border-b border-[#E6E8EB]",
                        li {
                            class: "me-2",
                            a {
                                class: "inline-block p-2 text-white bg-[#2f2640] rounded-t-lg active",
                                aria_current: "preview",
                                "Preview"
                            }
                        }
                        li {
                            class: "me-2",
                            a {
                                class: "inline-block p-2 rounded-t-lg hover:text-gray-600",
                                "AST"
                            }
                        }
                    },
                    div {
                        class: "p-2",
                        div {
                            for img_path in img_sources {
                                img {
                                    class: "text-zinc-800",
                                    src: "{img_path}",
                                }
                            }
                        }
                        div {
                            class: "p-2",
                            pre {
                                code {
                                    CodeHighlight{snippet: CodeSnippet{text: Rc::new(query)}}
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}


#[derive(Clone, Debug, PartialEq)]
pub struct CodeSnippet<'a> {
    pub text: Rc<&'a str>,
}

#[component]
pub fn CodeHighlight<'a>(cx: Scope, snippet: CodeSnippet<'a>) -> Element {
    let each_line: Vec<&str> = snippet.text.split("\n").collect();

    cx.render(rsx! {
        div {
            for line in each_line.into_iter() {
                rsx!(WordSpan{line: line})
            }
        }
    })
}

fn LeftPaddedSpace(token: &str) -> &str {
    let keyword = vec!["CREATE", ")", ");"];

    if keyword.contains(&token) {
        "pl-0"
    } else {
        "pl-2"
    }
}

#[component]
pub fn WordSpan<'a>(cx: Scope<'a>, line: &'a str) -> Element {
    let text_per_line: Vec<&str> = line.split_whitespace().collect();

    cx.render(rsx! {
        div {
            text_per_line.into_iter().map(|s| {
                rsx!( span {
                    class: ColorMarker(s),
                    span {
                        class: LeftPaddedSpace(s),
                        s
                    }
                    // s
                } span {" "} )
            })
        }
    })
}

fn ColorMarker(str_line: &str) -> &str {
    match str_line {
        "CREATE" | "TABLE" => "text-[#f60]",
        "PRIMARY" | "KEY" | "SERIAL" | "UNIQUE" => "text-[#603bb3]",
        "INT" | "VARCHAR" | "DATE" | "TIMESTAMP" => "text-[#0550ae]",
        "NULL" | "NOT NULL" => "text-[#07457c]",
        "{" | "}" => "#076678",
        "(" | ")" | "=>" | "&" => "text-[#faa356]",
        ";" | "," => "text-[#ccc]",
        _ => "text-[#000]",
    }
}

pub fn play_icon(cx: Scope) -> Element {
    cx.render(rsx!(
        svg { class: "mr-1",
            fill: "currentColor",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 16 16",
            width: "16",
            height: "16",
            path { 
                d: "m11.596 8.697-6.363 3.692c-.54.313-1.233-.066-1.233-.697V4.308c0-.63.692-1.01 1.233-.696l6.363 3.692a.802.802 0 0 1 0 1.393"
            }
        }
    ))
}

fn write_specs<P: AsRef<Path>>(data: &str, write_path: P) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(write_path)?;

    writeln!(&file, "{}", data)?;

    Ok(())
}

pub fn generate_d2(d2lang: String) -> String {
    let output_file = "public/d2lang/output.svg";

    let curr_dir = env::current_dir().unwrap();
    let write_path = curr_dir.join("public/d2lang/input.d2");
    write_specs(d2lang.as_str(), write_path).expect("Failed to write d2lang to file.");

    let input = "./public/d2lang/input.d2";
    let output = "./public/d2lang/output.svg";

    let mut cmd = Command::new("d2");
    let result = cmd.args([input, output]).stdin(Stdio::piped()).stdout(Stdio::piped());

    result.output().expect("Process failed");

    output_file.to_string()
}