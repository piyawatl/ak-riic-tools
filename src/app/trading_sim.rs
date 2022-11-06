#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
pub enum TradingPostPhase {
    L1,
    L2,
    L3,
}

impl ToString for TradingPostPhase {
    fn to_string(&self) -> String {
        match self {
            TradingPostPhase::L1 => "Lv1".to_string(),
            TradingPostPhase::L2 => "Lv2".to_string(),
            TradingPostPhase::L3 => "Lv3".to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Clone)]
pub enum TradingPostTailoringSkill {
    Alpha,
    Beta,
}

impl ToString for TradingPostTailoringSkill {
    fn to_string(&self) -> String {
        match self {
            TradingPostTailoringSkill::Alpha => "Alpha".to_string(),
            TradingPostTailoringSkill::Beta => "Beta".to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum HighRarityOperatorPhase {
    None,
    E0,
    E1,
    E2,
}

impl ToString for HighRarityOperatorPhase {
    fn to_string(&self) -> String {
        match self {
            HighRarityOperatorPhase::None => "None".to_string(),
            HighRarityOperatorPhase::E0 => "Elite 0".to_string(),
            HighRarityOperatorPhase::E1 => "Elite 1".to_string(),
            HighRarityOperatorPhase::E2 => "Elite 2".to_string(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TradingPostProductionInput {
    pub duration_minutes: i32,
    pub phase: TradingPostPhase,
    pub capacity: i32,
    pub speed100: i32,
    pub tailoring_ramped: Vec<(TradingPostTailoringSkill, i32)>,
    pub tequila_phase: HighRarityOperatorPhase,
    pub proviso_phase: HighRarityOperatorPhase,
    pub jaye_phase: HighRarityOperatorPhase,
}

impl Default for TradingPostProductionInput {
    fn default() -> Self {
        Self {
            duration_minutes: 720,
            phase: TradingPostPhase::L3,
            capacity: 10,
            speed100: 200,
            tailoring_ramped: vec![],
            tequila_phase: HighRarityOperatorPhase::None,
            proviso_phase: HighRarityOperatorPhase::None,
            jaye_phase: HighRarityOperatorPhase::None,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
#[serde(default)]
pub struct TradingPostProductionOutput {
    pub stall_chance: f64,
    pub average_stall_time: f64,
    pub total_lmd: f64,
    pub total_gold: f64,
    pub daily_lmd: f64,
    pub daily_gold: f64,
    pub net_lmd_speed: f64,
    pub net_gold_speed: f64,
}

impl Default for TradingPostProductionOutput {
    fn default() -> Self {
        Self {
            stall_chance: 0f64,
            average_stall_time: 0f64,
            total_lmd: 0f64,
            total_gold: 0f64,
            daily_lmd: 0f64,
            daily_gold: 0f64,
            net_lmd_speed: 0f64,
            net_gold_speed: 0f64,
        }
    }
}

pub fn simulate_tp_production(input: &TradingPostProductionInput) -> TradingPostProductionOutput {
    let tailoring_ramp_spec = match input.phase {
        TradingPostPhase::L1 => [(1.0, 1.0), (0.0, 0.0), (0.0, 0.0)],
        TradingPostPhase::L2 => [(0.7, 0.4), (0.3, 0.6), (0.0, 0.0)],
        TradingPostPhase::L3 => [(0.3, 0.05), (0.5, 0.1), (0.2, 0.85)],
    };

    let proviso_mod = match input.proviso_phase {
        HighRarityOperatorPhase::None => 0,
        HighRarityOperatorPhase::E0 => 1,
        HighRarityOperatorPhase::E1 => 1,
        HighRarityOperatorPhase::E2 => 2,
    };
    let bonus_lmd = [
        0.0,
        0.0,
        match input.tequila_phase {
            HighRarityOperatorPhase::None => 0.0,
            HighRarityOperatorPhase::E0 => 250.0,
            HighRarityOperatorPhase::E1 => 250.0,
            HighRarityOperatorPhase::E2 => 500.0,
        },
    ];

    let order_lmd = [1000 + 500 * proviso_mod, 1500 + 500 * proviso_mod, 2000];
    let order_gold = [-2 - proviso_mod, -3 - proviso_mod, -4];
    let order_duration = [144 * 60, 210 * 60, 276 * 60];
    let order_limit = input.capacity as usize;
    let sim_duration = (input.duration_minutes * 60) as usize;

    #[derive(Clone)]
    struct Cell {
        weight: f64,
        lmd: f64,
        gold: f64,
    }

    impl Default for Cell {
        fn default() -> Self {
            Self {
                weight: 0f64,
                lmd: 0f64,
                gold: 0f64,
            }
        }
    }

    let mut dp_table = vec![vec![Cell::default(); order_limit as usize]; sim_duration as usize + 1];

    fn highest_ramp(
        tailoring_ramped: &[(TradingPostTailoringSkill, i32)],
        elapsed_time: i32,
    ) -> f64 {
        tailoring_ramped
            .iter()
            .map(|(tailor, ramped)| {
                let ramped_progress = (ramped * 60 + elapsed_time) as f64 / 9000.0;
                let ramp_multiplier = match tailor {
                    TradingPostTailoringSkill::Alpha => 0.5,
                    TradingPostTailoringSkill::Beta => 1.0,
                };
                ramp_multiplier
                    * if ramped_progress > 1.0 {
                        1.0
                    } else {
                        ramped_progress
                    }
            })
            .reduce(|x, y| if x > y { x } else { y })
            .unwrap_or(0.0)
    }

    // first order
    {
        // use current order distribution for first partial order
        let current_ramp = highest_ramp(&input.tailoring_ramped, 0);
        let order_weight: Vec<f64> = tailoring_ramp_spec
            .iter()
            .map(|(base, peak)| base * (1.0 - current_ramp) + peak * current_ramp)
            .collect();
        for (otype, odur) in order_duration.iter().enumerate() {
            let mod_dur = (*odur as f64 * 100.0 / input.speed100 as f64).ceil() as usize;
            let weight = order_weight[otype] / mod_dur as f64;
            for (carried_time, dp_row) in dp_table.iter_mut().enumerate().take(mod_dur) {
                if carried_time < sim_duration {
                    // first order ends in sim duration
                    dp_row[0].weight += weight;
                    // tequila does not add LMD to first partial order
                    dp_row[0].lmd +=
                        order_lmd[otype] as f64 * weight * (carried_time as f64 / mod_dur as f64);
                    dp_row[0].gold +=
                        order_gold[otype] as f64 * weight * (carried_time as f64 / mod_dur as f64);
                } else {
                    // first order ends after sim duration
                    dp_row[0].weight += weight;
                    // tequila does not add LMD to first partial order
                    dp_row[0].lmd +=
                        order_lmd[otype] as f64 * weight * (sim_duration as f64 / mod_dur as f64);
                    dp_row[0].gold +=
                        order_gold[otype] as f64 * weight * (sim_duration as f64 / mod_dur as f64);
                }
            }
        }
    }

    let mut stalled_weight = 0.0;
    let mut stalled_lmd = 0.0;
    let mut stalled_gold = 0.0;
    let mut stalled_time = 0.0;
    // mid & last orders
    for t in 0..sim_duration {
        // non-capped orders
        for count in 0..(order_limit - 1) {
            if dp_table[t][count].weight > 0.0 {
                let current_ramp = highest_ramp(&input.tailoring_ramped, t as i32);
                let order_weight: Vec<f64> = tailoring_ramp_spec
                    .iter()
                    .map(|(base, peak)| base * (1.0 - current_ramp) + peak * current_ramp)
                    .collect();
                for (otype, odur) in order_duration.iter().enumerate() {
                    let mod_speed = match input.jaye_phase {
                        HighRarityOperatorPhase::E0 => input.speed100 - (4 * (count as i32 + 1)),
                        _ => input.speed100,
                    };
                    let mod_dur = ((odur * 100) as f64 / mod_speed as f64).ceil() as usize;
                    let tfinish = t + mod_dur;
                    let wfactor = order_weight[otype];
                    let combined_weight = wfactor * dp_table[t][count].weight;
                    if tfinish < sim_duration {
                        // mid orders
                        dp_table[tfinish][count + 1].weight += combined_weight;
                        dp_table[tfinish][count + 1].lmd += dp_table[t][count].lmd * wfactor
                            + order_lmd[otype] as f64 * combined_weight
                            + bonus_lmd[otype] * combined_weight;
                        dp_table[tfinish][count + 1].gold += dp_table[t][count].gold * wfactor
                            + order_gold[otype] as f64 * combined_weight;
                    } else {
                        // last order
                        dp_table[sim_duration][count + 1].weight += combined_weight;
                        // fully credit Tequila for last order even if only worked partially
                        dp_table[sim_duration][count + 1].lmd += dp_table[t][count].lmd * wfactor
                            + order_lmd[otype] as f64 * combined_weight * (sim_duration - t) as f64
                                / mod_dur as f64
                            + bonus_lmd[otype] * combined_weight;
                        dp_table[sim_duration][count + 1].gold += dp_table[t][count].gold * wfactor
                            + order_gold[otype] as f64
                                * combined_weight
                                * (sim_duration - t) as f64
                                / mod_dur as f64;
                    }
                }
            }
        }
        // capped orders
        let w = dp_table[t][order_limit - 1].weight;
        if w > 0.0 {
            stalled_weight += w;
            stalled_lmd += dp_table[t][order_limit - 1].lmd;
            stalled_gold += dp_table[t][order_limit - 1].gold;
            stalled_time += w * (sim_duration - t) as f64;
        }
    }
    let healthy_lmd: f64 = dp_table[sim_duration].iter().map(|x| x.lmd).sum();
    let healthy_gold: f64 = dp_table[sim_duration].iter().map(|x| x.gold).sum();
    let total_lmd = stalled_lmd + healthy_lmd;
    let total_gold = stalled_gold + healthy_gold;
    let lmd_24 = total_lmd * 1440.0 / input.duration_minutes as f64;
    let gold_24 = total_gold * 1440.0 / input.duration_minutes as f64;
    let gold_24_extra = gold_24 + lmd_24 / 500.0;
    let baseline_lmd = 1450.0 * 1440.0 / 203.4;
    let baseline_gold = 20.0;
    let net_tp_speed = lmd_24 / baseline_lmd;
    let net_gold_speed = gold_24_extra / baseline_gold;
    TradingPostProductionOutput {
        stall_chance: stalled_weight * 100.0,
        average_stall_time: stalled_time / 60.0,
        total_lmd,
        total_gold,
        daily_lmd: lmd_24,
        daily_gold: gold_24,
        net_lmd_speed: net_tp_speed * 100.0,
        net_gold_speed: net_gold_speed * 100.0,
    }
}
