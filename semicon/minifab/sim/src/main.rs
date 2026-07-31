use clap::Parser;
use minifab_sim::{run, Config, St, TICKS_PER_MIN, TICKS_PER_WEEK};

/// Deterministic-core DES for the Kempf 1994 Mini-Fab (comprehension
/// check; scope + exclusions in the crate docs and ../PATTERN-GAP-REPORT.md).
#[derive(Parser)]
#[command(name = "minifab", version)]
struct Args {
    /// simulated weeks
    #[arg(long, default_value_t = 52)]
    weeks: u64,
    /// weekly release mix: Test,A,B lots per week
    #[arg(long, default_value = "3,51,30", value_parser = parse_mix)]
    mix: (u64, u64, u64),
}

fn parse_mix(s: &str) -> Result<(u64, u64, u64), String> {
    let v: Vec<u64> = s
        .split(',')
        .map(|x| x.trim().parse().map_err(|e| format!("{e}")))
        .collect::<Result<_, _>>()?;
    if v.len() != 3 {
        return Err("mix must be Test,A,B".into());
    }
    Ok((v[0], v[1], v[2]))
}

fn main() {
    let args = Args::parse();
    let cfg = Config {
        per_week: args.mix,
        horizon: args.weeks * TICKS_PER_WEEK,
    };
    let r = run(&cfg);
    let (t, a, b) = args.mix;
    println!(
        "mini-fab deterministic core — {} weeks, mix {t}/{a}/{b} per week",
        args.weeks
    );
    println!("(machines+batching+setups+dedication+FIFO only; no personnel/transport/failures — see PATTERN-GAP-REPORT.md)");
    println!();
    println!("released           {:>10}", r.released);
    println!("completed          {:>10}", r.done);
    println!(
        "throughput         {:>10.2} lots/week",
        r.throughput_per_week()
    );
    println!("mean flow time     {:>10.1} min", r.mean_flow_min());
    println!("time-avg WIP       {:>10.2} lots", r.mean_wip());
    println!("end WIP            {:>10} lots", r.end_wip);
    for (i, ty) in ["Test", "A", "B"].iter().enumerate() {
        if r.done_by_type[i] > 0 {
            println!(
                "  {:<5} done {:>7}   mean flow {:>8.1} min",
                ty,
                r.done_by_type[i],
                r.flow_sum_by_type[i] as f64 / r.done_by_type[i] as f64 / TICKS_PER_MIN as f64
            );
        }
    }
    println!();
    println!("machine     setup%  handling%  processing%  occupied%");
    for (st, i, su, ha, pr) in &r.machine_kpis {
        let h = r.horizon as f64;
        println!(
            "{:<12}{:>5.2}   {:>6.2}     {:>6.2}      {:>6.2}   [{} m{}]",
            format!("{}{}", st.name().chars().next().unwrap(), i),
            *su as f64 / h * 100.0,
            *ha as f64 / h * 100.0,
            *pr as f64 / h * 100.0,
            (*su + *ha + *pr) as f64 / h * 100.0,
            st.name(),
            i
        );
    }
    println!(
        "max buffer occupancy: diff {} / impl {} / lith {} (spec caps 18/12/12; unbounded here)",
        r.max_buf[St::Diff as usize],
        r.max_buf[St::Impl as usize],
        r.max_buf[St::Lith as usize]
    );
}
