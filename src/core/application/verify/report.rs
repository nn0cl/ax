//! Verify result aggregate and summary text.

#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub archives_total: usize,
    pub archives_ok: usize,
    pub members_total: usize,
    pub members_checked: usize,
    pub members_skipped_no_hash: usize,
    pub empty_dirs_total: usize,
    /// Human-readable issues (empty ⇒ success).
    pub issues: Vec<String>,
    pub messages: Vec<String>,
}

impl VerifyReport {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn summary_line(&self) -> String {
        if self.is_ok() {
            format!(
                "verify ok: archives={}/{} members_checked={} members_total={} \
                 empty_dirs={} skipped_no_hash={} issues=0",
                self.archives_ok,
                self.archives_total,
                self.members_checked,
                self.members_total,
                self.empty_dirs_total,
                self.members_skipped_no_hash
            )
        } else {
            format!(
                "verify failed: issues={} archives_ok={}/{} members_checked={}/{} empty_dirs={}",
                self.issues.len(),
                self.archives_ok,
                self.archives_total,
                self.members_checked,
                self.members_total,
                self.empty_dirs_total
            )
        }
    }
}
