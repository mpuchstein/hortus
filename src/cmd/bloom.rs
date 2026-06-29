use crate::model::{load_all_beds, load_all_seeds, Garden, Seed};
use anyhow::{Context, Result};
use clap::Args;
use owo_colors::OwoColorize;
use pulldown_cmark::{html, Parser};
use std::collections::{BTreeMap, HashMap};
use std::fs;

/// Render the garden: a terminal mosaic, a static HTML bloom with a
/// force-directed link graph, and a regenerated front-gate index.md.
#[derive(Args, Debug)]
pub struct BloomArgs {
    /// Only print the terminal mosaic (skip html + index).
    #[arg(long)]
    pub terminal_only: bool,
}

pub fn run(args: BloomArgs) -> Result<()> {
    let garden = Garden::discover(None)?;
    let seeds = load_all_seeds(&garden)?;
    let beds = load_all_beds(&garden)?;

    if seeds.is_empty() {
        println!("{}", "nothing in bloom yet. plant a seed first.".yellow());
        return Ok(());
    }

    print_mosaic(&garden, &seeds, &beds);

    if !args.terminal_only {
        let html_out = render_bloom_html(&garden, &seeds, &beds)?;
        fs::write(garden.bloom_path(), html_out)
            .with_context(|| format!("writing {}", garden.bloom_path().display()))?;

        let index_out = render_index_md(&garden, &seeds, &beds);
        fs::write(garden.index_path(), index_out)
            .with_context(|| format!("writing {}", garden.index_path().display()))?;

        println!();
        println!(
            "  {} {}",
            "bloomed →".dimmed(),
            garden.bloom_path().display().to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "index   →".dimmed(),
            garden.index_path().display().to_string().bright_cyan()
        );
    }

    Ok(())
}

fn print_mosaic(garden: &Garden, seeds: &[Seed], beds: &[crate::model::Bed]) {
    let live = seeds.iter().filter(|s| !s.is_composted).count();
    let composted = seeds.len() - live;
    let header: String = if composted > 0 {
        format!("· {} live · {} composted · {} beds", live, composted, beds.len())
    } else {
        format!("· {} live · {} beds", live, beds.len())
    };
    println!();
    println!(
        "  {}  {}",
        "✿  hortus".bright_green().bold(),
        header.dimmed()
    );
    println!();

    // Header line: a tiny sparkline-like bar per day-of-life
    if !seeds.is_empty() {
        let first = seeds.first().unwrap().planted;
        let last = seeds.last().unwrap().planted;
        let span = (last - first).num_days().max(1) as usize + 1;
        let mut per_day: BTreeMap<String, usize> = BTreeMap::new();
        for s in seeds {
            *per_day.entry(s.planted.to_string()).or_default() += 1;
        }
        print!("    ");
        for i in 0..span {
            let day = first + chrono::Duration::days(i as i64);
            let count = per_day.get(&day.to_string()).copied().unwrap_or(0);
            let glyph = match count {
                0 => '·',
                1 => '▴',
                2..=3 => '❀',
                _ => '✿',
            };
            print!("{}", glyph.to_string().bright_green());
        }
        println!("  {}", format!("{} days", span).dimmed());
    }
    println!();

    // Beds summary
    if !beds.is_empty() {
        println!("  {}", "beds".bold());
        for b in beds {
            println!(
                "    {} {} {}",
                "❀".bright_cyan(),
                b.name.bright_cyan().bold(),
                format!("({})", b.seeds.len()).dimmed()
            );
        }
        println!();
    }

    // Moods histogram (if any)
    let mut moods: BTreeMap<String, usize> = BTreeMap::new();
    for s in seeds {
        if let Some(m) = &s.mood {
            *moods.entry(m.clone()).or_default() += 1;
        }
    }
    if !moods.is_empty() {
        println!("  {}", "moods".bold());
        for (m, c) in moods {
            let bar = "▮".repeat(c.min(20));
            println!(
                "    {:<14} {} {}",
                m.italic(),
                bar.bright_yellow(),
                format!("{}", c).dimmed()
            );
        }
        println!();
    }

    println!("  {}", garden.root.display().to_string().dimmed());
}

/// Render a self-contained bloom.html with a force-directed link graph.
fn render_bloom_html(
    garden: &Garden,
    seeds: &[Seed],
    beds: &[crate::model::Bed],
) -> Result<String> {
    // Build node + edge data for the force graph.
    // Nodes: one per seed.
    // Edges: two seeds share an edge if they appear in the same bed,
    //        or if they share a tag.
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for s in seeds {
        by_id.insert(s.id.clone(), nodes.len());
        let mut tags_label = s.tags.join(", ");
        if let Some(m) = &s.mood {
            if !tags_label.is_empty() {
                tags_label.push_str(" · ");
            }
            tags_label.push_str(&format!("mood: {}", m));
        }
        nodes.push(serde_json::json!({
            "id": s.id,
            "planted": s.planted.to_string(),
            "composted_at": s.composted_at.map(|d| d.to_string()),
            "epitaph": s.epitaph,
            "is_composted": s.is_composted,
            "body": s.body,
            "mood": s.mood,
            "tags": s.tags,
            "tags_label": tags_label,
        }));
    }

    let mut edges: Vec<serde_json::Value> = Vec::new();
    let mut seen: std::collections::HashSet<(String, String, String)> =
        std::collections::HashSet::new();

    for bed in beds {
        for i in 0..bed.seeds.len() {
            for j in (i + 1)..bed.seeds.len() {
                let a = &bed.seeds[i];
                let b = &bed.seeds[j];
                let (a, b) = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
                let key = (a.clone(), b.clone(), format!("bed:{}", bed.name));
                if seen.insert(key) {
                    edges.push(serde_json::json!({
                        "source": a, "target": b, "kind": "bed", "label": bed.name,
                    }));
                }
            }
        }
    }

    // Tag-based edges
    let mut tag_groups: HashMap<String, Vec<String>> = HashMap::new();
    for s in seeds {
        for t in &s.tags {
            tag_groups.entry(t.clone()).or_default().push(s.id.clone());
        }
    }
    for (tag, ids) in tag_groups {
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let a = &ids[i];
                let b = &ids[j];
                let (a, b) = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
                let key = (a.clone(), b.clone(), format!("tag:{}", tag));
                if seen.insert(key) {
                    edges.push(serde_json::json!({
                        "source": a, "target": b, "kind": "tag", "label": tag,
                    }));
                }
            }
        }
    }

    let nodes_json = serde_json::to_string(&nodes)?;
    let edges_json = serde_json::to_string(&edges)?;

    // Body summaries for the side panel (markdown -> html)
    let mut side_entries: Vec<String> = Vec::new();
    for s in seeds {
        let body_html = markdown_to_html(&s.body);
        let tags = if s.tags.is_empty() {
            String::new()
        } else {
            format!(
                "<div class=\"tags\">{}</div>",
                s.tags
                    .iter()
                    .map(|t| format!("<span class=\"tag\">{}</span>", html_escape(t)))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let mood = s
            .mood
            .as_deref()
            .map(|m| format!("<span class=\"mood\">{}</span>", html_escape(m)))
            .unwrap_or_default();
        let composted_class = if s.is_composted { " composted" } else { "" };
        let composted_marker = if s.is_composted {
            format!(
                " <span class=\"composted-tag\">composted {}</span>",
                s.composted_at.map(|d| d.to_string()).unwrap_or_default()
            )
        } else {
            String::new()
        };
        let epitaph_html = s
            .epitaph
            .as_deref()
            .map(|e| format!("<div class=\"epitaph\">\"{}\"</div>", html_escape(e)))
            .unwrap_or_default();
        side_entries.push(format!(
            "<article id=\"seed-{}\" class=\"seed{}\">\
               <header><span class=\"date\">{}</span> <span class=\"id\">{}</span>{} {} {}</header>\
               {}\
               <div class=\"body\">{}</div>\
             </article>",
            html_escape(&s.id),
            composted_class,
            s.planted,
            html_escape(&s.id),
            mood,
            tags,
            composted_marker,
            epitaph_html,
            body_html
        ));
    }

    let bed_chips = beds
        .iter()
        .map(|b| {
            format!(
                "<span class=\"bed-chip\">{} <em>{}</em></span>",
                html_escape(&b.name),
                b.seeds.len()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    let live = seeds.iter().filter(|s| !s.is_composted).count();
    let composted = seeds.len() - live;
    let counts = if composted > 0 {
        format!(
            "<span class=\"count\">{} seeds</span> · <span class=\"count\">{} composted</span> · <span class=\"count\">{} beds</span>",
            live, composted, beds.len()
        )
    } else {
        format!(
            "<span class=\"count\">{} seeds</span> · <span class=\"count\">{} beds</span>",
            seeds.len(),
            beds.len()
        )
    };

    let title = format!("hortus · bloom · {}", garden.root.display());

    // ----- Mood timeline data -----
    // Group seeds by day; pick the dominant mood of each day and its color.
    let mut by_day: std::collections::BTreeMap<chrono::NaiveDate, Vec<&Seed>> =
        std::collections::BTreeMap::new();
    for s in seeds.iter().filter(|s| !s.is_composted) {
        by_day.entry(s.planted).or_default().push(s);
    }
    let mood_data: Vec<serde_json::Value> = by_day
        .iter()
        .map(|(date, ss)| {
            let mut mood_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for s in ss {
                if let Some(m) = &s.mood {
                    *mood_counts.entry(m.clone()).or_default() += 1;
                }
            }
            let dominant = mood_counts
                .iter()
                .max_by_key(|(_, c)| *c)
                .map(|(m, _)| m.clone())
                .unwrap_or_default();
            serde_json::json!({
                "date": date.to_string(),
                "count": ss.len(),
                "mood": dominant,
                "color": mood_color(&dominant),
            })
        })
        .collect();
    let mood_json = serde_json::to_string(&mood_data)?;
    let mood_svg_placeholder = r#"<svg viewBox="0 0 800 70" preserveAspectRatio="none"></svg>"#;

    let template = include_str!("bloom_template.html");
    let rendered = template
        .replace("__TITLE__", &html_escape(&title))
        .replace("__NODES__", &nodes_json)
        .replace("__MOOD_DATA__", &mood_json)
        .replace("__MOOD_TIMELINE__", mood_svg_placeholder)
        .replace("__EDGES__", &edges_json)
        .replace("__SIDES__", &side_entries.join("\n"))
        .replace("__BEDS__", &bed_chips)
        .replace("__COUNTS__", &counts);
    Ok(rendered)
}

fn render_index_md(
    garden: &Garden,
    seeds: &[Seed],
    beds: &[crate::model::Bed],
) -> String {
    let mut out = String::new();
    out.push_str("# the garden, in passing\n\n");
    let live = seeds.iter().filter(|s| !s.is_composted).count();
    let composted = seeds.len() - live;
    let seed_count = if composted > 0 {
        format!("{} live seeds ({} composted)", live, composted)
    } else {
        format!("{} live seeds", live)
    };
    out.push_str(&format!(
        "*{} · {} beds · rooted at `{}`*\n\n",
        seed_count,
        beds.len(),
        garden.root.display()
    ));

    if !beds.is_empty() {
        out.push_str("## beds\n\n");
        for b in beds {
            out.push_str(&format!("- **{}** — {}", b.name, b.seeds.len()));
            if !b.description.is_empty() {
                out.push_str(&format!(" — {}", b.description));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    let climate = crate::model::Climate::load_or_default(garden).unwrap_or_default();
    if let Some(m) = climate.now.mood.as_deref() {
        out.push_str(&format!("**climate:** {}\n\n", m));
    }
    if let Some(r) = climate.now.reading.as_deref() {
        out.push_str(&format!("**reading:** {}\n\n", r));
    }

    out.push_str("## recent seeds\n\n");
    for s in seeds.iter().rev().take(10) {
        let prefix = if s.is_composted { "~ " } else { "" };
        out.push_str(&format!(
            "- {}**{}** — {}",
            prefix,
            s.planted,
            s.id,
        ));
        if let Some(m) = &s.mood {
            out.push_str(&format!(" · {}", m));
        }
        if !s.tags.is_empty() {
            out.push_str(&format!(" · {}", s.tags.join(", ")));
        }
        if s.is_composted {
            out.push_str(" · *composted*");
        }
        out.push('\n');
        if let Some(first) = s.body.lines().find(|l| !l.trim().is_empty()) {
            out.push_str(&format!("  > {}\n", first));
        }
        if let Some(e) = &s.epitaph {
            out.push_str(&format!("  > *epitaph: {}*\n", e));
        }
    }
    out.push('\n');

    out.push_str("---\n\n");
    out.push_str("*[open bloom.html for the living graph]*\n");
    out
}

fn markdown_to_html(md: &str) -> String {
    let parser = Parser::new(md);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Map a mood string to a color in the garden palette. Unknown moods get a
/// neutral moss.
fn mood_color(mood: &str) -> &'static str {
    let m = mood.to_lowercase();
    if m.contains("elated") || m.contains("joy") || m.contains("happy") {
        "#b6e3a5" // bright bloom green
    } else if m.contains("tender") || m.contains("soft") {
        "#d99a8a" // rose
    } else if m.contains("hopeful") {
        "#f4d35e" // gold
    } else if m.contains("curious") {
        "#88c5e2" // sky
    } else if m.contains("restless") {
        "#e89c4f" // amber
    } else if m.contains("quiet") || m.contains("still") {
        "#9eb1aa" // grey-green
    } else if m.contains("sad") || m.contains("grief") {
        "#7a8aa6" // dusk blue
    } else {
        "#5fb98a" // moss default
    }
}
