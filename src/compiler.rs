use crate::model::{ProjectState, VariableType};
use quote::quote;

pub struct Compiler;

impl Compiler {
    pub fn generate_cargo_toml(name: &str) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
egui = "0.33.3"
eframe = "0.33.3"
"#,
            name
        )
    }

    pub fn generate_main_rs() -> String {
        r#"#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
use app::MyApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Generated App",
        options,
        Box::new(|cc| {
             egui_extras::install_image_loaders(&cc.egui_ctx);
             Ok(Box::new(MyApp::default()))
        }),
    )
}
"#
        .to_string()
    }

    pub fn generate_app_rs(state: &ProjectState) -> String {
        // 1. Generate Struct Fields
        let mut fields = Vec::new();
        let mut inits = Vec::new();

        // Sort keys for deterministic output
        let mut keys: Vec<String> = state.variables.keys().cloned().collect();
        keys.sort();

        for key in keys {
            if let Some(var) = state.variables.get(&key) {
                let name = quote::format_ident!("{}", var.name);
                let val_str = &var.value; // Extract string reference
                let (ty, init_val) = match var.v_type {
                    VariableType::String => (quote! { String }, quote! { String::from(#val_str) }),
                    VariableType::Integer => (quote! { i32 }, {
                        let val: i32 = val_str.parse().unwrap_or(0);
                        quote! { #val }
                    }),
                    VariableType::Float => (quote! { f64 }, {
                        let val: f64 = val_str.parse().unwrap_or(0.0);
                        quote! { #val }
                    }),
                    VariableType::Boolean => (quote! { bool }, {
                        let val: bool = val_str.parse().unwrap_or(false);
                        quote! { #val }
                    }),
                };

                // Correction for String initialization
                let init_expr = if var.v_type == VariableType::String {
                    let s = &var.value;
                    quote! { #s.to_string() }
                } else {
                    init_val
                };

                fields.push(quote! { pub #name: #ty });
                inits.push(quote! { #name: #init_expr });
            }
        }

        // 2. Generate UI Code
        // This relies on WidgetNode::codegen() being updated to handle "self." prefixing
        let ui_body = state.root_node.codegen();

        let app_code = quote! {
            use eframe::App;
            use egui::Context;

            pub struct MyApp {
                #(#fields),*
            }

            impl Default for MyApp {
                fn default() -> Self {
                    Self {
                        #(#inits),*
                    }
                }
            }

            impl App for MyApp {
                fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        #ui_body
                    });
                }
            }
        };

        // Standard pretty-printing (basic)
        app_code.to_string()
    }
}
