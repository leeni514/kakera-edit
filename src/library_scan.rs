use crate::models::VisualNovel;
use std::cmp::Reverse;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
///how many folders deep to look for the game exe inside a vn folder
const MAX_EXECUTABLE_DEPTH: usize = 2;
///exes with these in their name are almost never the game itself
const IGNORED_EXECUTABLE_WORDS: &[&str] = &[
    "unins", "setup", "install", "config", "crash", "redist", "dxweb", "update", "7z",
];
///a game folder found inside the vn folder
#[derive(Debug, Clone, PartialEq)]
pub struct FoundVn {
    pub folder_path: String,
    pub folder_name: String,
    ///the folder name with junk like [tags] removed, used to search vndb
    pub search_title: String,
    pub executable_path: Option<String>,
}
///returns every subfolder of the vn folder that isn't already in the library
pub fn find_new_vns(vn_folder: &Path, library: &[VisualNovel]) -> Result<Vec<FoundVn>, io::Error> {
    let mut game_folders: Vec<PathBuf> = fs::read_dir(vn_folder)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    game_folders.sort();
    let mut found_vns = Vec::new();
    for game_folder in game_folders {
        let Some(folder_name) = game_folder.file_name() else {
            continue;
        };
        let folder_name = folder_name.to_string_lossy().to_string();
        if folder_name.starts_with('.') || folder_is_in_library(&game_folder, library) {
            continue;
        }
        found_vns.push(FoundVn {
            folder_path: game_folder.to_string_lossy().to_string(),
            search_title: clean_folder_name(&folder_name),
            executable_path: find_game_executable(&game_folder)
                .map(|path| path.to_string_lossy().to_string()),
            folder_name,
        });
    }
    Ok(found_vns)
}
///true when a vn in the library already points at this folder or an exe inside it
fn folder_is_in_library(game_folder: &Path, library: &[VisualNovel]) -> bool {
    library.iter().any(|vn| {
        let folder_matches = vn
            .game_folder
            .as_ref()
            .is_some_and(|folder| Path::new(folder) == game_folder);
        let executable_matches = vn
            .executable_path
            .as_ref()
            .is_some_and(|path| Path::new(path).starts_with(game_folder));
        folder_matches || executable_matches
    })
}
///turns "[Publisher] Some Game (v1.2)_EN" into "Some Game EN"
fn clean_folder_name(folder_name: &str) -> String {
    let mut cleaned = String::new();
    let mut bracket_depth = 0;
    for character in folder_name.chars() {
        match character {
            '[' | '(' | '{' => bracket_depth += 1,
            ']' | ')' | '}' => bracket_depth = (bracket_depth - 1).max(0),
            '_' if bracket_depth == 0 => cleaned.push(' '),
            _ if bracket_depth == 0 => cleaned.push(character),
            _ => {}
        }
    }
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        folder_name.to_string()
    } else {
        cleaned
    }
}
///picks the exe most likely to be the game: named like the folder, then least nested, then biggest
fn find_game_executable(game_folder: &Path) -> Option<PathBuf> {
    let folder_key = game_folder
        .file_name()
        .map(|name| simplify_name(&name.to_string_lossy()))
        .unwrap_or_default();
    let mut executables = Vec::new();
    collect_executables(game_folder, 0, &mut executables);
    executables
        .into_iter()
        .filter(|(path, _, _)| !is_ignored_executable(path))
        .max_by_key(|(path, depth, size)| {
            let executable_key = path
                .file_stem()
                .map(|stem| simplify_name(&stem.to_string_lossy()))
                .unwrap_or_default();
            let name_matches = !executable_key.is_empty()
                && (folder_key.contains(&executable_key) || executable_key.contains(&folder_key));
            (name_matches, Reverse(*depth), *size)
        })
        .map(|(path, _, _)| path)
}
///collects (path, depth, file size) for every exe under the folder
fn collect_executables(folder: &Path, depth: usize, executables: &mut Vec<(PathBuf, usize, u64)>) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if path.is_dir() {
            if depth < MAX_EXECUTABLE_DEPTH {
                collect_executables(&path, depth + 1, executables);
            }
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            executables.push((path, depth, size));
        }
    }
}
fn is_ignored_executable(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    IGNORED_EXECUTABLE_WORDS
        .iter()
        .any(|word| file_name.contains(word))
}
///lowercase letters and digits only, so "Steins;Gate" and "steinsgate" match
fn simplify_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleans_folder_names() {
        assert_eq!(clean_folder_name("[Nitroplus] Steins;Gate (v1.1)"), "Steins;Gate");
        assert_eq!(clean_folder_name("Fate_Stay_Night"), "Fate Stay Night");
        assert_eq!(clean_folder_name("[only tags]"), "[only tags]");
    }
    #[test]
    fn picks_the_game_executable() {
        let root = std::env::temp_dir().join(format!("kakera-scan-test-{}", std::process::id()));
        let game = root.join("Steins;Gate");
        fs::create_dir_all(game.join("tools")).unwrap();
        fs::write(game.join("unins000.exe"), vec![0; 5000]).unwrap();
        fs::write(game.join("tools").join("big_helper.exe"), vec![0; 9000]).unwrap();
        fs::write(game.join("SteinsGate.exe"), vec![0; 100]).unwrap();
        let found = find_new_vns(&root, &[]).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].executable_path.as_ref().unwrap().ends_with("SteinsGate.exe"));
    }
}
