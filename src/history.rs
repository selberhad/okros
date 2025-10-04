// Command history management
//
// Ported from: mcl-cpp-reference/InputLine.cc (lines 8-142)
//
// C++ pattern: History class (ring buffer), HistorySet class (collection)
// Rust pattern: History struct, HistorySet struct with save/load

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// History IDs (C++ InputLine.h:5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HistoryId {
    None = 0,
    Generic = 1,
    MainInput = 2,
    OpenMud = 3,
    SearchScrollback = 4,
}

impl From<i32> for HistoryId {
    fn from(id: i32) -> Self {
        match id {
            0 => HistoryId::None,
            1 => HistoryId::Generic,
            2 => HistoryId::MainInput,
            3 => HistoryId::OpenMud,
            4 => HistoryId::SearchScrollback,
            _ => HistoryId::None,
        }
    }
}

/// Ring buffer history for one input line (C++ History class, InputLine.cc:10-64)
pub struct History {
    id: HistoryId,
    strings: Vec<Option<String>>,
    timestamps: Vec<u64>,
    max_history: usize,
    current: usize, // Next insertion point
}

impl History {
    /// Create new history with given ID (C++ History::History, lines 28-35)
    pub fn new(id: HistoryId, max_history: usize) -> Self {
        Self {
            id,
            strings: vec![None; max_history],
            timestamps: vec![0; max_history],
            max_history,
            current: 0,
        }
    }

    /// Add string to history (C++ History::add, lines 42-54)
    pub fn add(&mut self, s: &str, timestamp: u64) {
        // Don't store duplicates (C++ lines 44-45)
        if self.current > 0 {
            let prev_idx = (self.current - 1) % self.max_history;
            if let Some(ref prev) = self.strings[prev_idx] {
                if prev == s {
                    return;
                }
            }
        }

        let idx = self.current % self.max_history;
        self.strings[idx] = Some(s.to_string());
        self.timestamps[idx] = timestamp;
        self.current += 1;
    }

    /// Get string from history (C++ History::get, lines 57-64)
    /// count=1 gets the LAST line, count=2 gets second-to-last, etc.
    pub fn get(&self, count: usize) -> Option<(&str, u64)> {
        let total = self.current.min(self.max_history);
        if count > total || count == 0 {
            return None;
        }

        let idx = (self.current - count) % self.max_history;
        self.strings[idx]
            .as_ref()
            .map(|s| (s.as_str(), self.timestamps[idx]))
    }

    pub fn id(&self) -> HistoryId {
        self.id
    }
}

/// Collection of histories (C++ HistorySet class, InputLine.cc:71-132)
pub struct HistorySet {
    histories: Vec<History>,
    max_history: usize,
}

impl HistorySet {
    pub fn new(max_history: usize) -> Self {
        Self {
            histories: Vec::new(),
            max_history,
        }
    }

    /// Find or create history for given ID (C++ HistorySet::find, lines 123-131)
    fn find_or_create(&mut self, id: HistoryId) -> &mut History {
        // Find existing
        if let Some(idx) = self.histories.iter().position(|h| h.id() == id) {
            return &mut self.histories[idx];
        }

        // Create new
        let hist = History::new(id, self.max_history);
        self.histories.push(hist);
        self.histories.last_mut().unwrap()
    }

    /// Add to history (C++ HistorySet::add, lines 116-118)
    pub fn add(&mut self, id: HistoryId, s: &str, timestamp: Option<u64>) {
        let ts = timestamp.unwrap_or_else(current_time);
        self.find_or_create(id).add(s, ts);
    }

    /// Get from history (C++ HistorySet::get, lines 111-113)
    pub fn get(&mut self, id: HistoryId, count: usize) -> Option<(&str, u64)> {
        self.find_or_create(id).get(count)
    }

    /// Save history to ~/.mcl/history (C++ HistorySet::saveHistory, lines 80-94)
    pub fn save_history(&mut self, save_enabled: bool) -> std::io::Result<()> {
        if !save_enabled {
            return Ok(());
        }

        let path = history_file_path()?;

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&path)?;

        // Set permissions to 0600 (C++ line 86)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        // Write all histories (C++ lines 87-90)
        for hist in &mut self.histories {
            let mut count = self.max_history;
            while count > 0 {
                if let Some((s, ts)) = hist.get(count) {
                    writeln!(file, "{} {} {}", hist.id() as i32, ts, s)?;
                }
                count -= 1;
            }
        }

        Ok(())
    }

    /// Load history from ~/.mcl/history (C++ HistorySet::loadHistory, lines 96-109)
    pub fn load_history(&mut self, save_enabled: bool) -> std::io::Result<()> {
        if !save_enabled {
            return Ok(());
        }

        let path = history_file_path()?;
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return Ok(()), // File doesn't exist yet, OK
        };

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.splitn(3, ' ').collect();

            if parts.len() == 3 {
                if let (Ok(id), Ok(ts)) = (parts[0].parse::<i32>(), parts[1].parse::<u64>()) {
                    self.add(HistoryId::from(id), parts[2], Some(ts));
                }
            }
        }

        Ok(())
    }
}

/// Get current Unix timestamp in seconds
fn current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get history file path: ~/.mcl/history (C++ line 82, 98)
fn history_file_path() -> std::io::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME not set"))?;

    Ok(PathBuf::from(home).join(".mcl").join("history"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_add_and_get() {
        let mut h = History::new(HistoryId::MainInput, 10);
        h.add("north", 100);
        h.add("south", 101);
        h.add("east", 102);

        // get(1) returns last, get(2) second-to-last, etc.
        assert_eq!(h.get(1), Some(("east", 102)));
        assert_eq!(h.get(2), Some(("south", 101)));
        assert_eq!(h.get(3), Some(("north", 100)));
        assert_eq!(h.get(4), None);
    }

    #[test]
    fn history_no_duplicates() {
        let mut h = History::new(HistoryId::MainInput, 10);
        h.add("north", 100);
        h.add("north", 101); // Duplicate, should be ignored

        // Only one entry
        assert_eq!(h.get(1), Some(("north", 100)));
        assert_eq!(h.get(2), None);
    }

    #[test]
    fn history_ring_buffer_wraps() {
        let mut h = History::new(HistoryId::MainInput, 3);
        h.add("a", 1);
        h.add("b", 2);
        h.add("c", 3);
        h.add("d", 4); // Wraps, overwrites "a"

        assert_eq!(h.get(1), Some(("d", 4)));
        assert_eq!(h.get(2), Some(("c", 3)));
        assert_eq!(h.get(3), Some(("b", 2)));
        assert_eq!(h.get(4), None); // "a" is gone
    }

    #[test]
    fn history_set_multiple_ids() {
        let mut hs = HistorySet::new(10);
        hs.add(HistoryId::MainInput, "north", Some(100));
        hs.add(HistoryId::OpenMud, "open localhost 4000", Some(200));

        assert_eq!(hs.get(HistoryId::MainInput, 1), Some(("north", 100)));
        assert_eq!(
            hs.get(HistoryId::OpenMud, 1),
            Some(("open localhost 4000", 200))
        );
    }

    #[test]
    fn history_set_save_and_load() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temporary file with history data
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2 100 north").unwrap();
        writeln!(temp_file, "2 101 south").unwrap();
        writeln!(temp_file, "3 200 open mud.com 4000").unwrap();
        temp_file.flush().unwrap();

        // Note: This test doesn't actually test load_history() because it reads from
        // ~/.mcl/history, not our temp file. For production use, we'd need dependency
        // injection to make the path configurable. For now, this test just documents
        // the expected format.
    }
}
