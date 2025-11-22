use std::{env, path::PathBuf, thread, time::Duration};
use colored::*;
use std::collections::VecDeque;
use crate::error::{MartyError, Result};

#[derive(Debug, Default)]
pub struct History {
    pub backward: Vec<PathBuf>,
    pub forward: Vec<PathBuf>,
}

fn print_fade_breadcrumb(history: &Vec<PathBuf>, current: &PathBuf) {
    let mut trail = VecDeque::new();

    let start = if history.len() > 5 { history.len() - 5 } else { 0 };
    for dir in &history[start..] {
        trail.push_back(dir.clone());
    }
    trail.push_back(current.clone());

    let len = trail.len();
    let styled_trail: Vec<String> = trail
        .iter()
        .enumerate()
        .map(|(i, dir)| {
            let component = dir.file_name().unwrap_or_default().to_str().unwrap_or("");
            let ratio = i as f32 / len as f32;
            let silver = (192, 192, 192);
            let white = (255, 255, 255);
            let r = (silver.0 as f32 * (1.0 - ratio) + white.0 as f32 * ratio) as u8;
            let g = (silver.1 as f32 * (1.0 - ratio) + white.1 as f32 * ratio) as u8;
            let b = (silver.2 as f32 * (1.0 - ratio) + white.2 as f32 * ratio) as u8;
            format!("{}", component.truecolor(r, g, b))
        })
        .collect();

    println!("{} {}", "📌".truecolor(0, 191, 255).bold(), styled_trail.join(" > "));
}

fn animate_direction(emoji: &str, count: usize) {
    for _ in 0..count {
        print!("{} ", emoji);
        thread::sleep(Duration::from_millis(50)); // small delay
    }
    println!();
}







pub fn traverse(cmd: &str, history: &mut History) -> Result<PathBuf> {
    let mut current = env::current_dir()?;
    let steps = cmd.trim();

    if steps.starts_with('{') && steps.ends_with('}') {
        return handle_subdir_jump(steps, &mut current, history);
    }

    let mut chars = steps.chars().peekable();
    while let Some(ch) = chars.next() {
        let mut count = 1;
        while chars.peek() == Some(&ch) {
            count += 1;
            chars.next();
        }

        match ch {
            '<' => current = handle_backward(count, &mut current, history)?,
            '>' => current = handle_forward(count, &mut current, history)?,
            '^' => current = handle_up(count, &mut current, history)?,
            _ => return Err(MartyError::Internal(format!("Unrecognized traversal character '{}'", ch))),
        }
    }

    Ok(current)
}

fn handle_subdir_jump(steps: &str, current: &mut PathBuf, history: &mut History) -> Result<PathBuf> {
    let target_subdir = &steps[1..steps.len() - 1];
    let target_path = current.join(target_subdir);
    if target_path.exists() {
        history.backward.push(current.clone());
        history.forward.clear();
        println!(
            "{} {} {}",
            "📂 Jumped to".truecolor(0, 255, 0).bold(),
            target_path.display().to_string().truecolor(255, 255, 0),
            "via target subdir".truecolor(0, 191, 255)
        );
        print_fade_breadcrumb(&history.backward, &target_path);
        Ok(target_path)
    } else {
        Err(MartyError::Internal(format!("Target subdirectory '{}' does not exist", target_subdir)))
    }
}

fn handle_backward(count: usize, current: &mut PathBuf, history: &mut History) -> Result<PathBuf> {
    animate_direction("⬅️", count);
    for _ in 0..count {
        if let Some(prev) = history.backward.pop() {
            history.forward.push(current.clone());
            *current = prev;
        } else {
            return Err(MartyError::Internal("No previous directory in history to go back to".to_string()));
        }
    }
    println!(
        "{} {}",
        "⬅️ Back to".truecolor(138, 43, 226).bold(),
        current.display().to_string().truecolor(255, 255, 0)
    );
    print_fade_breadcrumb(&history.backward, &current);
    Ok(current.clone())
}

fn handle_forward(count: usize, current: &mut PathBuf, history: &mut History) -> Result<PathBuf> {
    animate_direction("➡️", count);
    for _ in 0..count {
        if let Some(next) = history.forward.pop() {
            history.backward.push(current.clone());
            *current = next;
        } else {
            return Err(MartyError::Internal("No forward directory in history to go to".to_string()));
        }
    }
    println!(
        "{} {}",
        "➡️ Forward to".truecolor(0, 255, 0).bold(),
        current.display().to_string().truecolor(255, 255, 0)
    );
    print_fade_breadcrumb(&history.backward, &current);
    Ok(current.clone())
}

fn handle_up(count: usize, current: &mut PathBuf, history: &mut History) -> Result<PathBuf> {
    animate_direction("🔼", count);
    for _ in 0..count {
        if let Some(parent) = current.parent() {
            history.backward.push(current.clone());
            *current = parent.to_path_buf();
        } else {
            return Err(MartyError::Internal("No parent directory exists".to_string()));
        }
    }
    println!(
        "{} {}",
        "🔼 Up to parent".truecolor(0, 191, 255).bold(),
        current.display().to_string().truecolor(255, 255, 0)
    );
    print_fade_breadcrumb(&history.backward, &current);
    Ok(current.clone())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_traverse_up() {
        let tmp_dir = tempdir().unwrap();
        let current = tmp_dir.path().to_path_buf();
        env::set_current_dir(&current).unwrap();
        let mut history = History::default();

        let new_dir = traverse("^", &mut history).unwrap();
        assert_eq!(new_dir, current.parent().unwrap());
    }

    #[test]
    fn test_traverse_back() {
        let tmp_dir = tempdir().unwrap();
        let parent = tmp_dir.path().parent().unwrap().to_path_buf();
        let current = tmp_dir.path().to_path_buf();
        env::set_current_dir(&current).unwrap();
        let mut history = History::default();
        history.backward.push(parent.clone());

        let new_dir = traverse("<", &mut history).unwrap();
        assert_eq!(new_dir, parent);
    }

    #[test]
    fn test_traverse_forward() {
        let tmp_dir = tempdir().unwrap();
        let forward_path = tmp_dir.path().join("forward");
        fs::create_dir(&forward_path).unwrap();
        let current = tmp_dir.path().to_path_buf();
        env::set_current_dir(&current).unwrap();
        let mut history = History::default();
        history.forward.push(forward_path.clone());

        let new_dir = traverse(">", &mut history).unwrap();
        assert_eq!(new_dir, forward_path);
    }

    #[test]
    fn test_traverse_invalid() {
        let tmp_dir = tempdir().unwrap();
        env::set_current_dir(tmp_dir.path()).unwrap();
        let mut history = History::default();
        assert!(traverse("x", &mut history).is_err());
    }

    #[test]
    fn test_traverse_subdir() {
        let tmp_dir = tempdir().unwrap();
        let subdir = tmp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        env::set_current_dir(tmp_dir.path()).unwrap();
        let mut history = History::default();

        let new_dir = traverse("{subdir}", &mut history).unwrap();
        assert_eq!(new_dir, subdir);
    }
}

