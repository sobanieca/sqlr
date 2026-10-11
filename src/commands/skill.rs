use super::HELP_TEXT;
use crate::logger;
use crate::storage::home_dir;
use std::path::{Path, PathBuf};

const SKILL_TEMPLATE: &str = include_str!("../templates/SKILL.md");
const AGENT_INSTRUCTIONS: &str = include_str!("../templates/agent-instructions.txt");
const SKILL_TARGETS: &[(&str, &str)] = &[
    (".agents/skills", "Codex, Cursor"),
    (".claude/skills", "Claude Code, Cursor"),
];

pub fn execute(init: bool, connection: Option<String>) -> Result<(), String> {
    if init {
        return create_skill(connection.as_deref());
    }
    logger::print_line(AGENT_INSTRUCTIONS);
    logger::print_line(HELP_TEXT);
    Ok(())
}

fn skill_content(connection: Option<&str>) -> String {
    let (name, target, select) = match connection {
        Some(connection) => (
            format!("sqlr-{connection}"),
            format!("the `{connection}` database"),
            format!(
                "2. Run `sqlr set {connection}` first so every command targets this\n   connection."
            ),
        ),
        None => (
            "sqlr".to_string(),
            "a database".to_string(),
            "2. `sqlr ls` lists connections, the default is marked. Switch with\n   `sqlr set <name>`."
                .to_string(),
        ),
    };
    SKILL_TEMPLATE
        .replace("${name}", &name)
        .replace("${target}", &target)
        .replace("${select}", &select)
}

struct SkillRoot {
    dir: PathBuf,
    global: bool,
}

/// Inside a git repository (searched from cwd up to the home directory): the
/// git root. Outside a repository: the home directory.
fn find_skill_root() -> Result<SkillRoot, String> {
    let home = home_dir().map(PathBuf::from);
    let mut dir = std::env::current_dir().unwrap_or_default();
    loop {
        if dir.join(".git").exists() {
            return Ok(SkillRoot { dir, global: false });
        }
        if Some(&dir) == home.as_ref() || !dir.pop() {
            break;
        }
    }
    let home =
        home.ok_or("Not in a git repository and the home directory is unknown (set HOME).")?;
    Ok(SkillRoot {
        dir: home,
        global: true,
    })
}

fn create_skill(connection: Option<&str>) -> Result<(), String> {
    let skill_name = match connection {
        Some(connection) => format!("sqlr-{connection}"),
        None => "sqlr".to_string(),
    };
    let root = find_skill_root()?;
    let files: Vec<(PathBuf, &str)> = SKILL_TARGETS
        .iter()
        .map(|(dir, read_by)| (Path::new(dir).join(&skill_name).join("SKILL.md"), *read_by))
        .collect();

    for (path, _) in &files {
        let skill_file = root.dir.join(path);
        if skill_file.exists() {
            return Err(format!(
                "{} already exists. Delete it first if you want to regenerate it.",
                skill_file.display()
            ));
        }
    }

    let content = skill_content(connection);
    for (path, _) in &files {
        let skill_file = root.dir.join(path);
        if let Some(parent) = skill_file.parent() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        std::fs::write(&skill_file, &content).map_err(|err| err.to_string())?;
    }

    if root.global {
        logger::info(format!(
            "Not in a git repository - installing for all projects in {}",
            root.dir.display()
        ));
    } else {
        logger::info(format!("Project root: {} (git root)", root.dir.display()));
    }
    for (path, read_by) in &files {
        logger::info(format!("Created {}  ({read_by})", path.display()));
    }
    logger::info("");
    logger::info(format!(
        "Try: /{skill_name} how many users signed up this week"
    ));
    Ok(())
}
