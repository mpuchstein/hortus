use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Root of a garden on disk.
#[derive(Debug, Clone)]
pub struct Garden {
    pub root: PathBuf,
}

impl Garden {
    pub fn discover(explicit: Option<PathBuf>) -> Result<Self> {
        let root = if let Some(p) = explicit {
            Some(p)
        } else if let Ok(p) = std::env::var("HORTUS_ROOT") {
            Some(PathBuf::from(p))
        } else {
            None
        };
        let root = match root {
            Some(p) => p,
            None => {
                let cwd = std::env::current_dir()?;
                if cwd.join("seeds").is_dir() {
                    cwd
                } else if cwd.join("my-hortus").join("seeds").is_dir() {
                    cwd.join("my-hortus")
                } else {
                    cwd
                }
            }
        };
        let g = Garden { root };
        g.ensure_layout()?;
        Ok(g)
    }

    pub fn seeds_dir(&self) -> PathBuf {
        self.root.join("seeds")
    }
    pub fn beds_dir(&self) -> PathBuf {
        self.root.join("beds")
    }
    pub fn compost_dir(&self) -> PathBuf {
        self.root.join("compost")
    }
    pub fn climate_path(&self) -> PathBuf {
        self.root.join("climate.toml")
    }
    pub fn index_path(&self) -> PathBuf {
        self.root.join("index.md")
    }
    pub fn bloom_path(&self) -> PathBuf {
        self.root.join("bloom.html")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join(".hortus")
    }

    fn ensure_layout(&self) -> Result<()> {
        for d in [
            self.seeds_dir(),
            self.beds_dir(),
            self.compost_dir(),
            self.cache_dir(),
        ] {
            fs::create_dir_all(&d)
                .with_context(|| format!("creating directory {}", d.display()))?;
        }
        Ok(())
    }
}

/// A seed: a single thought, one file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seed {
    pub id: String,
    pub planted: NaiveDate,
    #[serde(default)]
    pub last_tended: Option<NaiveDate>,
    #[serde(default)]
    pub mood: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composted_at: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epitaph: Option<String>,
    /// The body of the seed. Stored in the markdown file below the
    /// frontmatter, so we skip serialization and default-deserialize.
    #[serde(default, skip_serializing)]
    pub body: String,
    /// Set at load time, derived from the file's location. Not serialized.
    #[serde(default, skip_serializing)]
    pub is_composted: bool,
}

impl Seed {
    pub fn file_path(&self, garden: &Garden) -> PathBuf {
        let dir = if self.is_composted {
            garden.compost_dir()
        } else {
            garden.seeds_dir()
        };
        dir.join(format!("{}.md", self.id))
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading seed {}", path.display()))?;
        let (fm, body) = split_frontmatter(&raw);
        let mut s: Seed =
            serde_yaml::from_str(&fm).with_context(|| format!("parsing {}", path.display()))?;
        s.body = body;
        s.is_composted = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some("compost");
        Ok(s)
    }

    pub fn save(&self, garden: &Garden) -> Result<()> {
        let new_path = self.file_path(garden);
        // If we're moving between seeds/ and compost/, remove the old file.
        let old_path_active = garden.seeds_dir().join(format!("{}.md", self.id));
        let old_path_compost = garden.compost_dir().join(format!("{}.md", self.id));
        if self.is_composted && old_path_active.exists() && old_path_active != new_path {
            let _ = fs::remove_file(&old_path_active);
        }
        if !self.is_composted && old_path_compost.exists() && old_path_compost != new_path {
            let _ = fs::remove_file(&old_path_compost);
        }
        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&serde_yaml::to_string(self).context("serializing seed")?);
        out.push_str("---\n\n");
        out.push_str(self.body.trim_start_matches('\n'));
        if !out.ends_with('\n') {
            out.push('\n');
        }
        fs::write(&new_path, out)
            .with_context(|| format!("writing {}", new_path.display()))?;
        Ok(())
    }
}

/// A bed: a thematic collection of seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bed {
    pub name: String,
    #[serde(default)]
    pub seeds: Vec<String>,
    #[serde(default)]
    pub description: String,
}

impl Bed {
    pub fn file_path(&self, garden: &Garden) -> PathBuf {
        garden.beds_dir().join(format!("{}.md", Bed::slug(&self.name)))
    }

    pub fn slug(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading bed {}", path.display()))?;
        let (fm, body) = split_frontmatter(&raw);
        let mut b: Bed =
            serde_yaml::from_str(&fm).with_context(|| format!("parsing {}", path.display()))?;
        if b.description.is_empty() {
            b.description = body.trim().to_string();
        }
        Ok(b)
    }

    pub fn save(&self, garden: &Garden) -> Result<()> {
        let path = self.file_path(garden);
        let mut out = String::new();
        out.push_str("---\n");
        out.push_str(&serde_yaml::to_string(self).context("serializing bed")?);
        out.push_str("---\n\n");
        if !self.description.is_empty() {
            out.push_str(&self.description);
            out.push('\n');
        }
        fs::write(&path, out).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

/// Climate: the ambient state of the gardener.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Climate {
    #[serde(default)]
    pub now: ClimateNow,
    #[serde(default)]
    pub history: Vec<ClimateSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClimateNow {
    #[serde(default)]
    pub mood: Option<String>,
    #[serde(default)]
    pub reading: Option<String>,
    #[serde(default)]
    pub season: Option<String>,
    #[serde(default)]
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateSnapshot {
    pub date: NaiveDate,
    pub mood: Option<String>,
    pub reading: Option<String>,
}

impl Climate {
    pub fn load_or_default(garden: &Garden) -> Result<Self> {
        let p = garden.climate_path();
        if p.exists() {
            let raw = fs::read_to_string(&p)
                .with_context(|| format!("reading climate {}", p.display()))?;
            Ok(toml::from_str(&raw).context("parsing climate.toml")?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, garden: &Garden) -> Result<()> {
        let p = garden.climate_path();
        let out = toml::to_string_pretty(self).context("serializing climate")?;
        fs::write(&p, out).with_context(|| format!("writing {}", p.display()))?;
        Ok(())
    }
}

/// All seeds in a garden, in memory. Includes composted seeds.
pub fn load_all_seeds(garden: &Garden) -> Result<Vec<Seed>> {
    let mut seeds = Vec::new();
    for dir in [garden.seeds_dir(), garden.compost_dir()] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir)
            .with_context(|| format!("reading {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                seeds.push(Seed::load(&path)?);
            }
        }
    }
    seeds.sort_by(|a, b| {
        a.planted
            .cmp(&b.planted)
            .then(a.is_composted.cmp(&b.is_composted))
            .then(a.id.cmp(&b.id))
    });
    Ok(seeds)
}

/// Only the live (not-composted) seeds.
pub fn load_live_seeds(garden: &Garden) -> Result<Vec<Seed>> {
    Ok(load_all_seeds(garden)?
        .into_iter()
        .filter(|s| !s.is_composted)
        .collect())
}

/// All beds in a garden, in memory.
pub fn load_all_beds(garden: &Garden) -> Result<Vec<Bed>> {
    let mut beds = Vec::new();
    for entry in fs::read_dir(garden.beds_dir())
        .with_context(|| format!("reading {}", garden.beds_dir().display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            beds.push(Bed::load(&path)?);
        }
    }
    beds.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(beds)
}

/// Split a markdown file into (frontmatter, body).
/// Returns ("", body) if there is no frontmatter.
pub fn split_frontmatter(raw: &str) -> (String, String) {
    let trimmed = raw.trim_start_matches('\n');
    if let Some(rest) = trimmed.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n") {
            let fm = rest[..end].to_string();
            let body = rest[end + 5..].to_string();
            return (fm, body);
        }
        if let Some(end) = rest.rfind("\n---") {
            let fm = rest[..end].to_string();
            let body = rest[end + 4..].to_string();
            return (fm, body);
        }
    }
    (String::new(), raw.to_string())
}

/// Build a seed id from today's date and a body excerpt.
pub fn make_seed_id(planted: NaiveDate, body: &str) -> String {
    let mut slug = String::new();
    for word in body.split_whitespace().take(4) {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>()
            .to_lowercase();
        if !cleaned.is_empty() {
            if !slug.is_empty() {
                slug.push('-');
            }
            slug.push_str(&cleaned);
        }
        if slug.len() >= 32 {
            break;
        }
    }
    if slug.is_empty() {
        slug = "untitled".to_string();
    }
    format!("{}-{}", planted.format("%Y-%m-%d"), slug)
}

/// Find a unique seed id, appending -2, -3, ... on collision.
pub fn unique_seed_id(garden: &Garden, base: &str) -> String {
    let mut candidate = base.to_string();
    let mut n = 2;
    while garden.seeds_dir().join(format!("{}.md", candidate)).exists() {
        candidate = format!("{}-{}", base, n);
        n += 1;
    }
    candidate
}
