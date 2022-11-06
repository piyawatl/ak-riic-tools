use egui::Ui;
use std::cmp::{max, min};
use uuid::Uuid;

use self::toggle_switch::toggle;
pub mod toggle_switch;
pub mod trading_sim;
use trading_sim::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TradingPostProductionApp {
    input: TradingPostProductionInput,
    output: TradingPostProductionOutput,
    is_editing: bool,
}

impl Default for TradingPostProductionApp {
    fn default() -> Self {
        Self {
            input: TradingPostProductionInput::default(),
            output: TradingPostProductionOutput::default(),
            is_editing: true,
        }
    }
}

impl TradingPostProductionApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TradingPostProductionApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            input,
            output,
            is_editing: _,
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TP calc");

            ui.horizontal(|ui| {
                ui.label("Edit Mode");
                ui.add(toggle(&mut self.is_editing));
                // ui.toggle_value(is_editing, "Edit Mode");
            });

            egui::Grid::new("tp calc")
            .num_columns(2)
            .spacing([4.0, 4.0])
            .show(ui, |ui| {

                // TP specs
                ui.group(|ui| {
                    egui::Grid::new("tp specs")
                        .num_columns(4)
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            // TP Level
                            ui.label("TP level: ");
                            if !&self.is_editing {
                                ui.label(input.phase.to_string());
                            } else {
                                egui::ComboBox::from_id_source("tp level")
                                    .selected_text(input.phase.to_string())
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_value(
                                                &mut input.phase,
                                                TradingPostPhase::L1,
                                                "Level 1",
                                            )
                                            .clicked()
                                        {
                                            input.capacity = 6;
                                        }
                                        if ui
                                            .selectable_value(
                                                &mut input.phase,
                                                TradingPostPhase::L2,
                                                "Level 2",
                                            )
                                            .clicked()
                                        {
                                            input.capacity = 8;
                                        }
                                        if ui
                                            .selectable_value(
                                                &mut input.phase,
                                                TradingPostPhase::L3,
                                                "Level 3",
                                            )
                                            .clicked()
                                        {
                                            input.capacity = 10;
                                        }
                                        // for (value, label) in TradingPostPhase::name_list().iter() {
                                        //     ui.selectable_value(&mut input.phase, value.clone(), label);
                                        // }
                                    });
                            }
                            ui.end_row();

                            // TP Order Limit
                            ui.label("TP order limit: ");
                            if !&self.is_editing {
                                ui.label(input.capacity.to_string());
                            } else {
                                ui.add(egui::Slider::new(&mut input.capacity, 0..=38));
                            }
                            ui.end_row();

                            // TP Speed
                            ui.label("TP speed: ");
                            if !&self.is_editing {
                                ui.label(input.speed100.to_string() + "%");
                            } else {
                                ui.add(egui::Slider::new(&mut input.speed100, 40..=300).suffix("%"));
                                ui.vertical_centered_justified(|ui| {
                                    if ui.button("+1").clicked() {
                                        input.speed100 = min(300, input.speed100 + 1);
                                    }
                                    if ui.button("-1").clicked() {
                                        input.speed100 = max(0, input.speed100 - 1);
                                    }
                                });
                            }
                            ui.end_row();

                            // Duration
                            ui.label("Duration: ");
                            if !&self.is_editing {
                                ui.label(
                                    format!("{:0>2}", input.duration_minutes / 60)
                                        + "h"
                                        + &*(format!("{:0>2}", input.duration_minutes % 60))
                                        + "m",
                                );
                            } else {
                                let label_hour = egui::Label::new(
                                    format!("{:0>2}", input.duration_minutes / 60) + "h",
                                );
                                let label_minute = egui::Label::new(
                                    format!("{:0>2}", input.duration_minutes % 60) + "m",
                                );
                                egui::Grid::new("shift duration")
                                    .num_columns(4)
                                    .spacing([4.0, 4.0])
                                    .show(ui, |ui| {
                                        let mut new_button = |ui: &mut egui::Ui, value, text: &str| {
                                            ui.centered_and_justified(|ui| {
                                                if ui.button(text).clicked() {
                                                    input.duration_minutes = max(
                                                        1,
                                                        min(5400, input.duration_minutes + value),
                                                    );
                                                }
                                            });
                                        };
                                        new_button(ui, 360, "+6h");
                                        new_button(ui, 10, "+10m");
                                        ui.end_row();

                                        new_button(ui, 60, "+1h");
                                        new_button(ui, 1, "+1m");
                                        ui.end_row();

                                        ui.centered_and_justified(|ui| ui.add(label_hour));
                                        ui.centered_and_justified(|ui| ui.add(label_minute));
                                        ui.end_row();

                                        new_button(ui, -60, "-1h");
                                        new_button(ui, -1, "-1m");
                                        ui.end_row();

                                        new_button(ui, -360, "-6h");
                                        new_button(ui, -10, "-10m");
                                        ui.end_row();
                                    });
                            }
                            ui.end_row();
                        });
                });
                // if ui.button("🔧").clicked() {}
                // if ui.button("✅").clicked() {}
                ui.end_row();

                // Tailoring Skills
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Tailoring skills:");
                    if !&self.is_editing {
                        if input.tailoring_ramped.is_empty() {
                            ui.label("(none)");
                        }
                        else {
                            egui::Grid::new("tailoring skills")
                                .num_columns(4)
                                .spacing([4.0, 4.0])
                                .show(ui, |ui| {
                                    for (skill, ramp, _id) in input.tailoring_ramped.iter_mut() {
                                        ui.label(skill.to_string());
                                        ui.label(
                                            format!("{:0>2}", *ramp / 60)
                                                + "h"
                                                + &*(format!("{:0>2}", *ramp % 60))
                                                + "m",
                                        );
                                        ui.end_row();
                                    }
                                });
                        }
                    } else {
                        egui::Grid::new("tailoring skills")
                            .num_columns(4)
                            .spacing([4.0, 4.0])
                            .show(ui, |ui| {
                                // ui.horizontal(|ui| {
                                input.tailoring_ramped.retain_mut(|(skill, ramp, id)| {
                                    let mut retained = true;
                                    let label_hour = egui::Label::new(
                                        format!("{:0>2}", *ramp / 60) + "h",
                                    );
                                    let label_minute = egui::Label::new(
                                        format!("{:0>2}", *ramp % 60) + "m",
                                    );
                                    ui.group(|ui| {
                                        ui.vertical_centered_justified(|ui| {
                                            if ui.button("❌").clicked() {
                                                retained = false;
                                            }
                                            egui::Grid::new(format!("{}ramped", &id))
                                                .num_columns(4)
                                                .spacing([4.0, 4.0])
                                                .show(ui, |ui| {
                                                    let mut new_button =
                                                        |ui: &mut egui::Ui, value, text: &str| {
                                                            ui.centered_and_justified(|ui| {
                                                                if ui.button(text).clicked() {
                                                                    *ramp =
                                                                        max(0, min(180, *ramp + value));
                                                                }
                                                            });
                                                        };
                                                    ui.label("");
                                                    new_button(ui, 180, "max");
                                                    new_button(ui, 10, "+10m");
                                                    ui.end_row();

                                                    ui.label("");
                                                    new_button(ui, 60, "+1h");
                                                    new_button(ui, 1, "+1m");
                                                    ui.end_row();

                                                    // ui.label("");
                                                    egui::ComboBox::from_id_source(&id)
                                                        .width(50.0)
                                                        .selected_text(skill.to_string())
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(
                                                                skill,
                                                                TradingPostTailoringSkill::Alpha,
                                                                "Alpha",
                                                            );
                                                            ui.selectable_value(
                                                                skill,
                                                                TradingPostTailoringSkill::Beta,
                                                                "Beta",
                                                            );
                                                        });
                                                    ui.centered_and_justified(|ui| ui.add(label_hour));
                                                    ui.centered_and_justified(|ui| {
                                                        ui.add(label_minute)
                                                    });
                                                    ui.end_row();

                                                    ui.label("");
                                                    new_button(ui, -60, "-1h");
                                                    new_button(ui, -1, "-1m");
                                                    ui.end_row();

                                                    ui.label("");
                                                    new_button(ui, -180, "min");
                                                    new_button(ui, -10, "-10m");
                                                    ui.end_row();
                                                });
                                        });
                                    });
                                    ui.end_row();
                                    retained
                                });
                                if input.tailoring_ramped.len() < 3 {
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("➕").clicked() {
                                            input.tailoring_ramped.push((
                                                TradingPostTailoringSkill::Alpha,
                                                0,
                                                Uuid::new_v4().as_u128(),
                                            ));
                                        }
                                    });
                                }
                            });
                    }
                    });
                });
                ui.end_row();
                // special skills
                ui.group(|ui| {
                    if !&self.is_editing {
                        egui::Grid::new("special skills - disp")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            // .striped(true)
                            .show(ui, |ui| {
                                ui.label("Jaye:");
                                ui.label(input.jaye_phase.to_string());
                                ui.end_row();
                                ui.label("Tequila:");
                                ui.label(input.tequila_phase.to_string());
                                ui.end_row();
                                ui.label("Proviso:");
                                ui.label(input.proviso_phase.to_string());
                                ui.end_row();
                            });
                    } else {
                        let op_combobox =
                            |ui: &mut egui::Ui, name, var: &mut HighRarityOperatorPhase| {
                                ui.label(name);
                                egui::ComboBox::from_id_source(name)
                                    .selected_text(var.to_string())
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(var, HighRarityOperatorPhase::None, "None");
                                        ui.selectable_value(
                                            var,
                                            HighRarityOperatorPhase::E0,
                                            "Elite 0",
                                        );
                                        ui.selectable_value(
                                            var,
                                            HighRarityOperatorPhase::E1,
                                            "Elite 1",
                                        );
                                        ui.selectable_value(
                                            var,
                                            HighRarityOperatorPhase::E2,
                                            "Elite 2",
                                        );
                                    });
                                ui.end_row();
                            };
                        egui::Grid::new("special skills")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            // .striped(true)
                            .show(ui, |ui| {
                                op_combobox(ui, "Jaye:", &mut input.jaye_phase);
                                op_combobox(ui, "Tequila:", &mut input.tequila_phase);
                                op_combobox(ui, "Proviso:", &mut input.proviso_phase);
                            });
                    }
                });
                ui.end_row();

            });

            //// Output
            ui.separator();
            if ui.button("Calculate").clicked() {
                *output = simulate_tp_production(input);
            }
            egui::Grid::new("output grid")
                .num_columns(4)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let right_align_label = |ui: &mut Ui, text: String| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(text);
                        });
                    };
                    ui.label("Stall chance");
                    right_align_label(ui, format!("{:.4}", output.stall_chance));
                    ui.label("%");
                    ui.end_row();
                    ui.label("Average stall time");
                    right_align_label(ui, format!("{:.4}", output.average_stall_time));
                    ui.label("minutes");
                    ui.end_row();
                    ui.label("Total LMD");
                    right_align_label(ui, format!("{:.4}", output.total_lmd));
                    ui.end_row();
                    ui.label("Total Gold");
                    right_align_label(ui, format!("{:.4}", output.total_gold));
                    ui.end_row();
                    ui.label("Daily LMD");
                    right_align_label(ui, format!("{:.4}", output.daily_lmd));
                    ui.end_row();
                    ui.label("Daily Gold");
                    right_align_label(ui, format!("{:.4}", output.daily_gold));
                    ui.end_row();
                    ui.label("Net LMD Speed");
                    right_align_label(ui, format!("{:.4}", output.net_lmd_speed));
                    ui.label("%");
                    ui.end_row();
                    ui.label("Net Gold Speed");
                    right_align_label(ui, format!("{:.4}", output.net_gold_speed));
                    ui.label("%");
                    ui.end_row();
                });
            // ui.horizontal(|ui| ui.label(""));
        });

        if false {
            egui::CentralPanel::default().show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's

                ui.heading("Arknight RIIC Tools");
                ui.hyperlink("https://github.com/emilk/eframe_template");
                ui.add(egui::github_link_file!(
                    "https://github.com/emilk/eframe_template/blob/master/",
                    "Source code."
                ));
                egui::warn_if_debug_build(ui);
            });
        }

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}
