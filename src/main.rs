use jsonrpc_core::futures::Future;
use jsonrpc_core::serde::Deserialize;
use jsonrpc_core::*;
use lsp_types::*;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Deserialize)]
struct Identifiable {
    id: String,
}

fn main() {
    let mut io = IoHandler::new();
    let mut log_file = File::create(&"/Users/anthonybullard/.gleamlsp").unwrap();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out_handle = stdout.lock();
    let mut in_handle = stdin.lock();

    io.add_method("initialize", |_: Params| {
        let result: InitializeResult = InitializeResult {
            capabilities: lsp_types::ServerCapabilities {
                text_document_sync: None,
                selection_range_provider: None,
                hover_provider: Some(true),
                completion_provider: None,
                signature_help_provider: None,
                definition_provider: None,
                type_definition_provider: None,
                implementation_provider: None,
                references_provider: None,
                document_highlight_provider: None,
                document_symbol_provider: None,
                workspace_symbol_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: None,
                document_link_provider: None,
                color_provider: None,
                folding_range_provider: None,
                declaration_provider: None,
                execute_command_provider: None,
                workspace: None,
                experimental: None,
            },
            server_info: None,
        };
        if let Ok(json) = serde_json::to_value(&result) {
            Ok(json)
        } else {
            Ok(Value::String("Oops".into()))
        }
    });

    io.add_method("textDocument/hover", |p: Params| {
        let response = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "# Hello\nThis is something".to_string(),
            }),
            range: None,
        };
        if let Ok(json) = serde_json::to_value(&response) {
            Ok(json)
        } else {
            Err(Error {
                code: ErrorCode::ParseError,
                message: "".to_string(),
                data: None,
            })
        }
    });

    io.add_notification("initialized", |_| ());

    loop {
        if let Ok(message) = parse_transport_message(&mut in_handle) {
            log_file
                .write_all(format!("{}\n", message).as_bytes())
                .unwrap_or(());
            if let Some(response) = io.handle_request_sync(&message) {
                log_file.write_all("Handled\n".as_bytes()).unwrap_or(());
                write_transport_message(&response, &mut out_handle).unwrap_or(());
            } else {
                log_file.write_all("Cant handle\n".as_bytes()).unwrap_or(());
                let params = ShowMessageRequestParams {
                    typ: lsp_types::MessageType::Warning,
                    message: "Can't Handle this message".to_string(),
                    actions: Some(vec![MessageActionItem {
                        title: "OK".to_string(),
                    }]),
                };
                if let Ok(json) = serde_json::to_string(&params) {
                    write_method_message("window/showMessageRequest", 1000, &json, &mut out_handle)
                        .unwrap_or(());
                } else {
                    log_file
                        .write_all("Cant send message\n".as_bytes())
                        .unwrap_or(());
                }
            }
        } else {
            log_file.write_all("Cant parse\n".as_bytes()).unwrap_or(());
        }
    }
}

const CONTENT_LENGTH: &'static str = "Content-Length:";

pub fn parse_transport_message<R: io::BufRead + Sized>(
    reader: &mut R,
) -> std::result::Result<String, String> {
    let mut content_length: u32 = 0;
    loop {
        let mut line = String::new();
        if let Err(_) = reader.read_line(&mut line) {
            break;
        }
        if line.starts_with(CONTENT_LENGTH) {
            let len_str: &str = &line[CONTENT_LENGTH.len()..];
            let int_result = len_str.trim().parse::<u32>().map_err(|_| {
                (String::from(CONTENT_LENGTH) + " not defined or invalid.").to_string()
            })?;
            content_length = int_result;
        } else if line.eq("\r\n") {
            break;
        } else if line.is_empty() {
            return Err("End of stream reached.".to_string());
        }
    }
    if content_length == 0 {
        return Err(String::from(CONTENT_LENGTH) + " not defined or invalid.");
    }
    let mut message_reader = reader.take(content_length as u64);
    let mut message = String::new();
    message_reader
        .read_to_string(&mut message)
        .map_err(|_| "".to_string())?;
    return Ok(message);
}

pub fn write_method_message<WRITE: io::Write>(
    method: &str,
    id: usize,
    message: &str,
    out: &mut WRITE,
) -> std::result::Result<(), ()> {
    //    let out : &mut io::Write = out;
    let buf = String::new();
    let method_message = format!(
        "{{\"jsonrpc\": \"2.0\", \"method\": \"{}\", \"id\": {}, \"params\": {}}}",
        method,
        id.to_string(),
        message
    );
    write_transport_message(&method_message, out)
}

pub fn write_transport_message<WRITE: io::Write>(
    message: &str,
    out: &mut WRITE,
) -> std::result::Result<(), ()> {
    //    let out : &mut io::Write = out;
    out.write_all(CONTENT_LENGTH.as_bytes()).map_err(|_| ())?;
    out.write(&[' ' as u8]).map_err(|_| ())?;
    let contents = message.as_bytes();
    out.write_all(contents.len().to_string().as_bytes())
        .map_err(|_| ())?;
    out.write_all("\r\n\r\n".as_bytes()).map_err(|_| ())?;
    out.write_all(message.as_bytes()).map_err(|_| ())?;
    out.flush().map_err(|_| ())?;
    Ok(())
}

trait SourceTree<E> {
    fn insert(&self) -> std::result::Result<(), E>;
    fn update(&self) -> std::result::Result<(), E>;
    fn symbol(&self) -> std::result::Result<(), E>;
}

#[derive(Debug, PartialEq)]
enum GleamLspError {}
impl std::fmt::Display for GleamLspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

struct NoopSourceTree {}
impl SourceTree<GleamLspError> for NoopSourceTree {
    fn insert(&self) -> std::result::Result<(), GleamLspError> {
        Ok(())
    }
    fn update(&self) -> std::result::Result<(), GleamLspError> {
        Ok(())
    }
    fn symbol(&self) -> std::result::Result<(), GleamLspError> {
        Ok(())
    }
}
